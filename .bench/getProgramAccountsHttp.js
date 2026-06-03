const BASE_URL = "http://localhost:4000";

async function getProgramAccounts(payload) {
  const res = await fetch(BASE_URL, {
    method: "POST",
    headers: {
      "Content-Type": "application/json",
    },
    body: JSON.stringify(payload),
  });

  return res;
}

async function getSlot() {
  const res = await fetch(BASE_URL, {
    method: "POST",
    headers: {
      "Content-Type": "application/json",
    },
    body: JSON.stringify({
      jsonrpc: "2.0",
      id: 1,
      method: "getSlot",
      params: [{ commitment: "confirmed" }],
    }),
  });

  return res;
}

async function main() {
  const slot = await getSlot();
  console.log(await slot.json());

  const startTime = new Date().getTime();

  const res = await getProgramAccounts({
    jsonrpc: "2.0",
    id: 1,
    method: "getProgramAccounts",
    params: [
      "Stake11111111111111111111111111111111111111",
      {
        encoding: "base64",
        commitment: "confirmed",
        filters: [
          {
            memcmp: {
              offset: 44,
              bytes: "eucZ4sCpD4McS9xGnSTZFAEw7iznGFJXKaQZKyecHu1",
            },
          },
          // {
          //   memcmp: {
          //     offset: 44,
          //     bytes: "5xoBq7f7CDgZwqHrDBdRWM84ExRetg4gZq93dyJtoSwp",
          //   },
          // },
        ],
      },
    ],
  });

  const endTime = new Date().getTime();
  const duration = endTime - startTime;
  const data = await res.json();
  console.log(data.result.length);
  console.log(`Query took: ${duration}ms`);
}

main().catch(console.error);
