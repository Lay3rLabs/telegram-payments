/**
 * Neutron blockchain service
 */

const NEUTRON_RPC = "https://neutron-rpc.publicnode.com:443";
const NEUTRON_DENOM = "untrn"; // Micro NTRN (1 NTRN = 1,000,000 untrn)

export interface NeutronBalance {
  amount: string;
  denom: string;
  formatted: string; // Human readable amount in NTRN
}

type Balance = {
  amount: string;
  denom: string;
};
/**
 * Fetch Neutron balance for an address
 */
export async function getNeutronBalance(
  address: string
): Promise<NeutronBalance> {
  try {
    const response = await fetch(
      `${NEUTRON_RPC}/cosmos/bank/v1beta1/balances/${address}`
    );

    if (!response.ok) {
      // Account doesn't exist on chain yet, return 0 balance
      return {
        amount: "0",
        denom: NEUTRON_DENOM,
        formatted: "0 NTRN",
      };
    }

    const data = await response.json();

    // Find NTRN balance
    const ntrnBalance = data.balances?.find(
      (b: Balance) => b.denom === NEUTRON_DENOM
    );

    if (!ntrnBalance) {
      return {
        amount: "0",
        denom: NEUTRON_DENOM,
        formatted: "0 NTRN",
      };
    }

    // Convert micro NTRN to NTRN
    const amountInMicroNtrn = parseInt(ntrnBalance.amount);
    const amountInNtrn = amountInMicroNtrn / 1_000_000;

    return {
      amount: ntrnBalance.amount,
      denom: NEUTRON_DENOM,
      formatted: `${amountInNtrn.toFixed(6)} NTRN`,
    };
  } catch (error) {
    console.log("Error fetching Neutron balance:", error);
    // Network error or other issue - return 0 balance
    return {
      amount: "0",
      denom: NEUTRON_DENOM,
      formatted: "0 NTRN",
    };
  }
}
