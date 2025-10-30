/**
 * Contract Configuration
 * Update these values with your deployed contract addresses
 */

export interface ContractConfig {
  address: string;
  codeId?: number;
}

export interface NetworkConfig {
  chainId: string;
  chainName: string;
  rpcEndpoint: string;
  restEndpoint: string;
  prefix: string;
  denom: string;
  decimals: number;
}

// Network configurations
export const NETWORKS: Record<string, NetworkConfig> = {
  "neutron-1": {
    chainId: "neutron-1",
    chainName: "Neutron",
    rpcEndpoint: "https://neutron-rpc.publicnode.com:443",
    restEndpoint: "https://neutron-rest.publicnode.com:443",
    prefix: "neutron",
    denom: "untrn",
    decimals: 6,
  },
  "pion-1": {
    chainId: "pion-1",
    chainName: "Neutron Testnet",
    rpcEndpoint: "https://rpc-palvus.pion-1.ntrn.tech",
    restEndpoint: "https://rest-palvus.pion-1.ntrn.tech",
    prefix: "neutron",
    denom: "untrn",
    decimals: 6,
  },
};

// Contract addresses by network
export const CONTRACTS: Record<string, Record<string, ContractConfig>> = {
  "neutron-1": {
    payments: {
      // TODO: Adjust to deployed contract address
      address: "neutron13nj4jrt88cs594fcga4q60qfzk4akwm7k3wph4",
      codeId: undefined,
    },
  },
};

// Get current network (can be overridden via env var or local storage)
export function getCurrentNetwork(): string {
  // Check localStorage first
  const stored = localStorage.getItem("selected_network");
  if (stored && NETWORKS[stored]) {
    return stored;
  }

  // Default to mainnet
  return "neutron-1";
}

// Get network config
export function getNetworkConfig(networkId?: string): NetworkConfig {
  const network = networkId || getCurrentNetwork();
  const config = NETWORKS[network];

  if (!config) {
    throw new Error(`Network ${network} not found in configuration`);
  }

  return config;
}

// Get contract address
export function getContractAddress(
  contractName: string,
  networkId?: string
): string {
  const network = networkId || getCurrentNetwork();
  const contract = CONTRACTS[network]?.[contractName];

  if (!contract) {
    throw new Error(
      `Contract ${contractName} not found for network ${network}`
    );
  }

  if (contract.address === "neutron1...") {
    throw new Error(
      `Contract ${contractName} address not configured. Please update src/config/contracts.ts with the deployed contract address.`
    );
  }

  return contract.address;
}

// Set contract address (useful for development)
export function setContractAddress(
  contractName: string,
  address: string,
  networkId?: string
): void {
  const network = networkId || getCurrentNetwork();

  if (!CONTRACTS[network]) {
    CONTRACTS[network] = {};
  }

  CONTRACTS[network][contractName] = {
    address,
  };

  // Also save to localStorage for persistence
  localStorage.setItem(`contract_${network}_${contractName}`, address);
}

// Load contract addresses from localStorage (for development)
export function loadContractAddressesFromStorage(): void {
  Object.keys(NETWORKS).forEach((network) => {
    Object.keys(CONTRACTS[network] || {}).forEach((contractName) => {
      const stored = localStorage.getItem(
        `contract_${network}_${contractName}`
      );
      if (stored) {
        CONTRACTS[network][contractName].address = stored;
      }
    });
  });
}

// Initialize on import
loadContractAddressesFromStorage();
