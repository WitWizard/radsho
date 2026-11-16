import type { ReactNode } from "react";
import { Toggle } from "./ui";

interface SettingRowProps {
  title: string;
  hint?: string;
  checked: boolean;
  onChange: (value: boolean) => void;
}

export function SettingRow({ title, hint, checked, onChange }: SettingRowProps) {
  return (
    <div className="flex items-center justify-between gap-4 py-3">
      <div className="min-w-0">
        <div className="text-[15px] font-semibold text-fg">{title}</div>
        {hint && <div className="mt-0.5 text-[13px] leading-snug text-fg2">{hint}</div>}
      </div>
      <Toggle checked={checked} onChange={onChange} label={title} />
    </div>
  );
}

interface FieldProps {
  label: string;
  children: ReactNode;
}

export function Field({ label, children }: FieldProps) {
  return (
    <label className="block py-2">
      <span className="mb-1.5 block text-[13px] font-semibold text-fg2">{label}</span>
      {children}
    </label>
  );
}

export const inputClass =
  "w-full rounded-2xl border border-line bg-chip px-4 py-2.5 text-[14px] text-fg outline-none placeholder:text-fg2 focus:border-mint";
