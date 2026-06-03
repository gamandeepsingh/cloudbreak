// SPDX-License-Identifier: AGPL-3.0-only
/*
 * Copyright 2025-2026 Triton One Limited. All rights reserved.
 */

use clap::{Parser, Subcommand};
use serde::Deserialize;
use std::fs;

pub mod analytics;
pub mod config;

#[derive(Deserialize, Debug, Clone)]
pub struct ServerConfig {
    pub name: String,
    pub database_url: String,
}

#[derive(Deserialize, Debug, Clone)]
pub struct Config {
    pub servers: Vec<ServerConfig>,
}

#[derive(clap::Parser, Debug)]
#[command(name = "dbtools")]
pub struct Cli {
    /// Path to the configuration file
    #[arg(short, long, default_value = "cloudbreak.dbtools.toml")]
    pub config: String,

    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Run analytics queries
    Analytics {
        #[command(subcommand)]
        command: AnalyticsCommands,
    },
    /// Modify the database configuration
    Config {
        #[command(subcommand)]
        command: ConfigCommands,
    },
}

#[derive(Subcommand, Debug)]
pub enum AnalyticsCommands {
    /// Get the biggest programs by account count
    GetBiggestPrograms,
    /// Get the size of the tables
    TableSize,
    /// Get the sizes of the indexes
    IndexesSizes,
    /// Get the sizes of the partitions
    PartitionSizes,
    /// Get the distinct owners of the accounts
    DistinctOwnersCount,
    /// Get the amount of mint accounts for token and token 2022
    MintAccountsCount,
    /// Get the count of the indexes in the database
    IndexesCount,
    /// Get the slow queries in the database
    SlowQueries,
    /// Get the delegates in the database
    GetDelegates,
    /// Get the row count for accounts and snapshot_accounts
    AccountsCount,
}

#[derive(Subcommand, Debug)]
pub enum ConfigCommands {
    /// Sets the level of detail for pg_tracing spans (both flags are false by default, so if the disable is received, it will be disabled
    ///  we can ignore it and use the default `false` value of the enable flag)
    PgTracingHighDetail {
        #[arg(long = "true", action = clap::ArgAction::SetTrue, conflicts_with = "disable")]
        enable: bool,
        #[arg(long = "false", action = clap::ArgAction::SetTrue, conflicts_with = "enable")]
        disable: bool,
    },
}

fn print_server_header(name: &str) {
    let border = "=".repeat(60);
    println!("\n{border}");
    println!("  SERVER: {name}");
    println!("{border}\n");
}

#[tokio::main]
async fn main() {
    let args = Cli::parse();

    let config = fs::read_to_string(args.config).expect("Failed to read config file");
    let config: Config = toml::from_str(&config).expect("Failed to parse config file");

    for server in &config.servers {
        print_server_header(&server.name);

        match &args.command {
            Commands::Analytics { command } => match command {
                AnalyticsCommands::GetBiggestPrograms => {
                    analytics::get_biggest_programs(&server.database_url).await
                }
                AnalyticsCommands::TableSize => {
                    analytics::get_table_size(&server.database_url).await
                }
                AnalyticsCommands::IndexesSizes => {
                    analytics::get_indexes_sizes(&server.database_url).await
                }
                AnalyticsCommands::PartitionSizes => {
                    analytics::get_partition_sizes(&server.database_url).await
                }
                AnalyticsCommands::DistinctOwnersCount => {
                    analytics::get_distinct_owners_count(&server.database_url).await
                }
                AnalyticsCommands::MintAccountsCount => {
                    analytics::get_mint_accounts_count(&server.database_url).await
                }
                AnalyticsCommands::IndexesCount => {
                    analytics::get_indexes_count(&server.database_url).await
                }
                AnalyticsCommands::SlowQueries => {
                    analytics::get_slow_queries(&server.database_url).await
                }
                AnalyticsCommands::GetDelegates => {
                    analytics::get_delegates(&server.database_url).await
                }
                AnalyticsCommands::AccountsCount => {
                    analytics::get_accounts_count(&server.database_url).await
                }
            },
            Commands::Config { command } => match command {
                ConfigCommands::PgTracingHighDetail { enable, disable: _ } => {
                    config::pg_tracing::pg_tracing_high_detail(&server.database_url, *enable).await
                }
            },
        }
    }
}
