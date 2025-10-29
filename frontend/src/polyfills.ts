/**
 * Browser polyfills for Node.js APIs
 */

import { Buffer } from 'buffer';

// Make Buffer available globally
(window as any).Buffer = Buffer;
(window as any).global = window;
