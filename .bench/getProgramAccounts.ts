import http from "k6/http";
import tempo from "https://jslib.k6.io/http-instrumentation-tempo/1.0.1/index.js";
import { check, group } from "k6";

const BASE_URL: string = __ENV.TARGET_URL || "http://localhost:8899";

const HEADERS: Record<string, string> = {
  "Content-Type": "application/json",
};

interface Payload {
  jsonrpc: string;
  id: number;
  method: string;
  params: [string, Record<string, any>];
}

interface JsonResponse {
  result?: any[];
}

function getProgramAccounts(payload: Payload): void {
  const programId: string = payload.params[0];
  const res = http.post(BASE_URL, JSON.stringify(payload), {
    headers: HEADERS,
  });

  check(res, {
    [`Program ID: ${programId} is status 200`]: (r) => r.status === 200,
  });
}

export const options: Record<string, any> = {
  scenarios: {
    main: {
      executor: "constant-arrival-rate",
      rate: Number(__ENV.K6_SCENARIO_RATE) || 10,
      timeUnit: __ENV.K6_SCENARIO_TIME_UNIT || "1s",
      duration: __ENV.K6_SCENARIO_DURATION || "1m",
      preAllocatedVUs: Number(__ENV.K6_SCENARIO_PRE_ALLOCATED_VUS) || 500,
      maxVUs: Number(__ENV.K6_SCENARIO_MAX_VUS) || 10000,
    },
  },
};

tempo.instrumentHTTP({
  propagator: "w3c",
});

export default function () {
  group("Solend Token Accounts", () => {
    getProgramAccounts({
      jsonrpc: "2.0",
      id: 1,
      method: "getProgramAccounts",
      params: [
        "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA",
        {
          encoding: "base64",
          commitment: "confirmed",
          filters: [
            { dataSize: 165 },
            {
              memcmp: {
                offset: 0,
                bytes: "SLNDpmoWTVADgEdndyvWzroNL7zSi1dF9PC3xHGtPwp",
              },
            },
          ],
        },
      ],
    });
  });

  group("Hyper Token Accounts", () => {
    getProgramAccounts({
      jsonrpc: "2.0",
      id: 1,
      method: "getProgramAccounts",
      params: [
        "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA",
        {
          encoding: "base64",
          commitment: "confirmed",
          filters: [
            { dataSize: 165 },
            {
              memcmp: {
                offset: 0,
                bytes: "Aq8Gocyvyyi8xk5EYxd6viUfVmVvs9T9R6mZFzZFpump",
              },
            },
          ],
        },
      ],
    });
  });
}
