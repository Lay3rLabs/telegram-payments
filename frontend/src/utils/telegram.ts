/**
 * Telegram Web App Utilities
 * Provides version-safe wrappers for Telegram WebApp methods
 */

import WebApp from '@twa-dev/sdk';

/**
 * Shows an alert with fallback for older versions
 * Requires Telegram Bot API 6.2+
 */
export function showAlert(message: string, callback?: () => void): void {
  if (WebApp.isVersionAtLeast('6.2')) {
    WebApp.showAlert(message, callback);
  } else {
    // Fallback to browser alert
    alert(message);
    if (callback) callback();
  }
}

/**
 * Shows a confirmation dialog with fallback for older versions
 * Requires Telegram Bot API 6.2+
 */
export function showConfirm(message: string, callback: (confirmed: boolean) => void): void {
  if (WebApp.isVersionAtLeast('6.2')) {
    WebApp.showConfirm(message, callback);
  } else {
    // Fallback to browser confirm
    const result = confirm(message);
    callback(result);
  }
}

/**
 * Shows a popup with custom buttons and fallback for older versions
 * Requires Telegram Bot API 6.2+
 */
export function showPopup(
  params: {
    title?: string;
    message: string;
    buttons?: Array<{ id?: string; type?: string; text?: string }>;
  },
  callback?: (buttonId: string) => void
): void {
  if (WebApp.isVersionAtLeast('6.2')) {
    WebApp.showPopup(params, callback);
  } else {
    // Fallback to browser alert
    const message = params.title ? `${params.title}\n\n${params.message}` : params.message;
    alert(message);
    if (callback) callback('ok');
  }
}

/**
 * Triggers haptic feedback with version check
 * Requires Telegram Bot API 6.1+
 */
export function hapticImpact(style: 'light' | 'medium' | 'heavy' | 'rigid' | 'soft'): void {
  if (WebApp.isVersionAtLeast('6.1') && WebApp.HapticFeedback) {
    WebApp.HapticFeedback.impactOccurred(style);
  }
}

/**
 * Triggers notification haptic feedback with version check
 * Requires Telegram Bot API 6.1+
 */
export function hapticNotification(type: 'error' | 'success' | 'warning'): void {
  if (WebApp.isVersionAtLeast('6.1') && WebApp.HapticFeedback) {
    WebApp.HapticFeedback.notificationOccurred(type);
  }
}

/**
 * Get the Telegram Web App version
 */
export function getTelegramVersion(): string {
  return WebApp.version || '6.0';
}

/**
 * Check if a specific feature is supported
 */
export function isFeatureSupported(feature: 'popup' | 'haptic' | 'settings' | 'main_button'): boolean {
  switch (feature) {
    case 'popup':
      return WebApp.isVersionAtLeast('6.2');
    case 'haptic':
      return WebApp.isVersionAtLeast('6.1');
    case 'settings':
      return WebApp.isVersionAtLeast('6.10');
    case 'main_button':
      return WebApp.isVersionAtLeast('6.1');
    default:
      return false;
  }
}
