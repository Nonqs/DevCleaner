import { sum, multiply, percent } from "./math";
import { logInfo, logDebug } from "./utils/logger";
import "./side-effects";

const total = sum(10, 20);
logInfo(`Total: ${total}`);

const tax = percent(total, 10);
console.log(`Tax: ${tax}`);

// Intentionally unused imports:
// - multiply
// - logDebug
