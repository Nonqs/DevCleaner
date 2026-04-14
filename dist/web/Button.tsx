import { useMemo } from "react";
import { formatLabel } from "./format";
type ButtonProps = {
    label: string;
};
export function Button({ label }: ButtonProps) {
    const text = useMemo(()=>formatLabel(label), [
        label
    ]);
    return <button>{text}</button>;
}
