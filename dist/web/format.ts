export function formatLabel(label: string): string {
    return label.trim().toUpperCase();
}
export function unusedFormat(label: string): string {
    return `unused:${label}`;
}
