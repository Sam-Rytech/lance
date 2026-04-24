import { StellarWalletsKit, Networks } from "@creit.tech/stellar-wallets-kit";

let kit: StellarWalletsKit | null = null;

export function getWalletsKit(): StellarWalletsKit {
  if (!kit) {
    kit = new StellarWalletsKit({
      network:
        (process.env.NEXT_PUBLIC_STELLAR_NETWORK as Networks) ??
        Networks.TESTNET,
      selectedWalletId: "freighter",
    });
  }
  return kit;
}

export async function connectWallet(): Promise<string> {
  if (process.env.NEXT_PUBLIC_E2E === "true") return "GD...CLIENT";
  const walletsKit = getWalletsKit();
  return new Promise<string>((resolve, reject) => {
    walletsKit.openModal({
      onWalletSelected: async () => {
        try {
          walletsKit.closeModal();
          const { address } = await walletsKit.getAddress();
          resolve(address);
        } catch (err) {
          reject(err);
        }
      },
    });
  });
}

export async function getConnectedWalletAddress(): Promise<string | null> {
  if (process.env.NEXT_PUBLIC_E2E === "true") return "GD...CLIENT";
  try {
    const { address } = await getWalletsKit().getAddress();
    return address ?? null;
  } catch {
    return null;
  }
}

export async function signTransaction(xdr: string): Promise<string> {
  if (process.env.NEXT_PUBLIC_E2E === "true") return xdr;
  const walletsKit = getWalletsKit();
  const networkPassphrase =
    (process.env.NEXT_PUBLIC_STELLAR_NETWORK as Networks) ?? Networks.TESTNET;
  const { signedTxXdr } = await walletsKit.signTransaction(xdr, {
    networkPassphrase,
  });
  return signedTxXdr;
}

/**
 * Signs a plaintext SIWS message via the connected wallet.
 * Returns a base64-encoded signature string.
 */
export async function signMessage(message: string): Promise<string> {
  if (process.env.NEXT_PUBLIC_E2E === "true") {
    return Buffer.from("e2e-mock-signature").toString("base64");
  }
  const walletsKit = getWalletsKit();
  const { signedMessage } = await walletsKit.signMessage(message);
  return Buffer.from(signedMessage).toString("base64");
}