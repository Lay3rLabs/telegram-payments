import { useEffect } from 'react';
import './WalletConnect.css';

export function WalletConnect() {
  // Get URI and wallet type from URL query parameters
  const params = new URLSearchParams(window.location.search);
  const uri = params.get('uri');
  const wallet = params.get('wallet') || 'keplr';

  const walletName = wallet === 'keplr' ? 'Keplr' : wallet === 'leap' ? 'Leap' : 'Wallet';

  // Generate deeplink
  let deeplink: string;
  if (wallet === 'keplr') {
    deeplink = `keplrwallet://wcV2?${uri}`;
  } else if (wallet === 'leap') {
    deeplink = `leapcosmos://wcV2?${uri}`;
  } else {
    deeplink = uri || '';
  }

  useEffect(() => {
    if (uri && deeplink) {
      // Automatically try to open the wallet
      console.log(`Opening ${wallet}:`, deeplink);
      window.location.href = deeplink;
    }
  }, [uri, wallet, deeplink]);

  if (!uri) {
    return (
      <div className="wallet-connect-page">
        <div className="connect-container">
          <p style={{ color: 'var(--tg-theme-hint-color)' }}>Invalid connection link</p>
        </div>
      </div>
    );
  }

  return (
    <div className="wallet-connect-page">
      <div className="connect-container">
        <div className="spinner"></div>
        <p className="status-text">Opening {walletName}...</p>
        <button
          className="open-wallet-button"
          onClick={() => window.location.href = deeplink}
        >
          Open {walletName}
        </button>
      </div>
    </div>
  );
}
