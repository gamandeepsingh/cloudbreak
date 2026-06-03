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
  params: [string, Record<string, any>, Record<string, any>];
}

interface JsonResponse {
  result?: any[];
}

function getTokenAccountsByOwner(payload: Payload): void {
  const owner: string = payload.params[0];
  const res = http.post(BASE_URL, JSON.stringify(payload), {
    headers: HEADERS,
  });

  check(res, {
    [`TokenAccount Owner: ${owner} is status 200`]: (r) => r.status === 200,
  });
}

export const options: Record<string, any> = {
  scenarios: {
    main: {
      executor: "constant-arrival-rate",
      rate: Number(__ENV.K6_SCENARIO_RATE) || 10,
      timeUnit: __ENV.K6_SCENARIO_TIME_UNIT || "1s",
      duration: __ENV.K6_SCENARIO_DURATION || "30s",
      preAllocatedVUs: Number(__ENV.K6_SCENARIO_PRE_ALLOCATED_VUS) || 500,
      maxVUs: Number(__ENV.K6_SCENARIO_MAX_VUS) || 10000,
    },
  },
};

tempo.instrumentHTTP({
  propagator: "w3c",
});

export default function () {
  group("Owner : 53..kx", () => {
    getTokenAccountsByOwner({
      jsonrpc: "2.0",
      id: 1,
      method: "getTokenAccountsByOwner",
      params: [
        "53eUvY6nyuMLKYazJGJzDWptKHMEgUD2FKxueGpEpHkx",
        {},
        {
          encoding: "jsonParsed",
          commitment: "confirmed",
        },
      ],
    });
  });

  group("Owner : As..Zt, mint : BONK", () => {
    getTokenAccountsByOwner({
      jsonrpc: "2.0",
      id: 1,
      method: "getTokenAccountsByOwner",
      params: [
        "AsM97N16ejpKcVJTwEWtnLsDMz7jFPGr6SU1vzJD9xZt",
        {
          mint: "AsM97N16ejpKcVJTwEWtnLsDMz7jFPGr6SU1vzJD9xZt",
        },
        {
          encoding: "base64",
          commitment: "confirmed",
        },
      ],
    });
  });

  group("Owner : 8r..4f, program : Legacy Token Program", () => {
    getTokenAccountsByOwner({
      jsonrpc: "2.0",
      id: 1,
      method: "getTokenAccountsByOwner",
      params: [
        "8rrBaEqmiWbb9JzLHePuA9zX4ToHWoxC4U2KTbebFt4f",
        {
          programId: "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA",
        },
        {
          commitment: "confirmed",
        },
      ],
    });
  });
}
