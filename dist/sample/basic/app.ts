import { sum, percent } from "./math";
import { logInfo } from "./utils/logger";
const total = sum(10, 20);
logInfo(`Total: ${total}`);
const tax = percent(total, 10);
console.log(`Tax: ${tax}`);
