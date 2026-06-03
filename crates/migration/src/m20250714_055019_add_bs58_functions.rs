// SPDX-License-Identifier: AGPL-3.0-only
/*
 * Copyright 2025-2026 Triton One Limited. All rights reserved.
 */

use sea_orm_migration::{
    prelude::*,
    sea_orm::{DatabaseBackend, Statement},
};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // bs58_decode function
        manager
            .get_connection()
            .execute(Statement::from_string(
                DatabaseBackend::Postgres,
                r#"
                CREATE OR REPLACE FUNCTION bs58_decode(
                    encoded text
                ) RETURNS bytea AS $$
                DECLARE
                    alphabet char(58) := '123456789ABCDEFGHJKLMNPQRSTUVWXYZabcdefghijkmnopqrstuvwxyz';
                    output bytea := '';
                    num numeric := 0;
                    c char(1);
                    p int;
                    byte bytea;
                BEGIN
                    FOR i IN 1..char_length(encoded) LOOP
                        c := substring(encoded FROM i FOR 1);
                        p := position(c IN alphabet);
                        IF p = 0 THEN
                            RAISE 'Illegal base58 character ''%'' in ''%''', c, encoded;
                        END IF;
                        num := (num * 58) + (p - 1);
                    END LOOP;

                    WHILE num > 0 LOOP
                        p := mod(num, 256);
                        byte := decode(lpad(to_hex(p), 2, '0'), 'hex');
                        output := byte || output;
                        num := (num - p) / 256;
                    END LOOP;

                    RETURN output;
                END;
                $$ LANGUAGE plpgsql IMMUTABLE;
                "#,
            ))
            .await?;

        // bs58_encode function (no length check)
        manager
            .get_connection()
            .execute(Statement::from_string(
                DatabaseBackend::Postgres,
                r#"
                CREATE OR REPLACE FUNCTION bs58_encode(
                    input bytea
                ) RETURNS text AS $$
                DECLARE
                    alphabet char(58) := '123456789ABCDEFGHJKLMNPQRSTUVWXYZabcdefghijkmnopqrstuvwxyz';
                    output text := '';
                    num numeric := 0;
                    p int;
                    b int;
                    i int;
                BEGIN
                    IF input IS NULL THEN
                        RETURN NULL;
                    END IF;

                    FOR i IN 0..(length(input) - 1) LOOP
                        b := get_byte(input, i);
                        num := num * 256 + b;
                    END LOOP;

                    WHILE num >= 58 LOOP
                        p := mod(num, 58);
                        output := substring(alphabet FROM (p + 1)::int FOR 1) || output;
                        num := (num - p) / 58;
                    END LOOP;

                    IF num > 0 THEN
                        output := substring(alphabet FROM (num + 1)::int FOR 1) || output;
                    END IF;

                    -- Preserve leading zeros
                    FOR i IN 0..(length(input) - 1) LOOP
                        IF get_byte(input, i) = 0 THEN
                            output := substring(alphabet FROM 1 FOR 1) || output;
                        ELSE
                            EXIT;
                        END IF;
                    END LOOP;

                    RETURN output;
                END;
                $$ LANGUAGE plpgsql IMMUTABLE;
                "#,
            ))
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .get_connection()
            .execute(Statement::from_string(
                DatabaseBackend::Postgres,
                r#"DROP FUNCTION IF EXISTS bs58_decode(text);"#,
            ))
            .await?;

        manager
            .get_connection()
            .execute(Statement::from_string(
                DatabaseBackend::Postgres,
                r#"DROP FUNCTION IF EXISTS bs58_encode(bytea);"#,
            ))
            .await?;

        Ok(())
    }
}
