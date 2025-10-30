import { useEffect, useState } from "react";
import WebApp from "@twa-dev/sdk";
import { Onboarding } from "./components/Onboarding";
import { RegisteredDashboard } from "./components/RegisteredDashboard";
import { useSigningClient } from "./hooks/useSigningClient";
import "./App.css";

// Track registration completion in localStorage
const REGISTRATION_KEY = "telegram_payments_registered";

function isUserRegistered(): boolean {
  return localStorage.getItem(REGISTRATION_KEY) === "true";
}

function setUserRegistered(registered: boolean) {
  if (registered) {
    localStorage.setItem(REGISTRATION_KEY, "true");
  } else {
    localStorage.removeItem(REGISTRATION_KEY);
  }
}

function App() {
  const [isRegistered, setIsRegistered] = useState(false);
  const [isLoading, setIsLoading] = useState(true);

  // Check wallet connection status
  const { client: signingClient, isReady: signerReady } = useSigningClient();

  useEffect(() => {
    // Initialize Telegram WebApp
    WebApp.ready();
    WebApp.expand();

    // Apply Telegram theme
    document.documentElement.style.setProperty(
      "--tg-theme-bg-color",
      WebApp.backgroundColor
    );
    document.documentElement.style.setProperty(
      "--tg-theme-text-color",
      WebApp.themeParams.text_color || "#000000"
    );
    document.documentElement.style.setProperty(
      "--tg-theme-hint-color",
      WebApp.themeParams.hint_color || "#999999"
    );
    document.documentElement.style.setProperty(
      "--tg-theme-button-color",
      WebApp.themeParams.button_color || "#3390ec"
    );
    document.documentElement.style.setProperty(
      "--tg-theme-button-text-color",
      WebApp.themeParams.button_text_color || "#ffffff"
    );
    document.documentElement.style.setProperty(
      "--tg-theme-secondary-bg-color",
      WebApp.themeParams.secondary_bg_color || "#f0f0f0"
    );

    // Check registration status
    setIsRegistered(isUserRegistered());
    setIsLoading(false);
  }, []);

  const handleOnboardingComplete = () => {
    setUserRegistered(true);
    setIsRegistered(true);
  };

  const handleLogout = () => {
    setIsRegistered(false);
    setUserRegistered(false);
  };

  if (isLoading || !signerReady) {
    return (
      <div className="loading-screen">
        <div className="loading-spinner">💸</div>
        <p>Loading...</p>
      </div>
    );
  }

  // Step 1: Not registered or wallet not connected - show onboarding (includes account creation)
  if (!isRegistered || !signingClient) {
    return <Onboarding onComplete={handleOnboardingComplete} />;
  }

  // Step 2: Registered user with connected wallet - show main dashboard
  return <RegisteredDashboard onLogout={handleLogout} />;
}

export default App;
