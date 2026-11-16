import { useEffect, useRef, useState, type ReactNode } from "react";
import { AnimatePresence, motion } from "framer-motion";
import { Check, ChevronDown, Star, X } from "lucide-react";
import { cn } from "../utils/cn";
import { useApp } from "../state/AppContext";

interface ToggleProps {
  checked: boolean;
  onChange: (value: boolean) => void;
  label?: string;
}

export function Toggle({ checked, onChange, label }: ToggleProps) {
  return (
    <button
      type="button"
      role="switch"
      aria-checked={checked}
      aria-label={label}
      onClick={() => onChange(!checked)}
      className={cn(
        "relative h-[26px] w-[48px] shrink-0 rounded-full border-0 transition-colors duration-200",
        checked ? "bg-mint" : "bg-[#4b5654]",
      )}
    >
      <span
        className={cn(
          "absolute top-[3px] h-5 w-5 rounded-full bg-[#0e1514] transition-all duration-200",
          checked ? "start-[25px]" : "start-[3px]",
        )}
      />
    </button>
  );
}

interface SegmentedProps<T extends string> {
  value: T;
  options: Array<{ value: T; label: string }>;
  onChange: (value: T) => void;
  small?: boolean;
}

export function Segmented<T extends string>({ value, options, onChange, small = false }: SegmentedProps<T>) {
  return (
    <div className="inline-flex rounded-full border border-line bg-chip p-[3px]">
      {options.map((option) => (
        <button
          key={option.value}
          type="button"
          onClick={() => onChange(option.value)}
          className={cn(
            "rounded-full border-0 font-semibold transition-colors",
            small ? "px-3 py-1 text-xs" : "px-4 py-1.5 text-[13px]",
            value === option.value ? "bg-mint text-[#06251a]" : "bg-transparent text-fg2 hover:text-fg",
          )}
        >
          {option.label}
        </button>
      ))}
    </div>
  );
}

export interface SelectOption<T extends string> {
  value: T;
  label: string;
  sublabel?: string;
  icon?: ReactNode;
}

interface CustomSelectProps<T extends string> {
  value: T;
  options: Array<SelectOption<T>>;
  onChange: (value: T) => void;
}

export function CustomSelect<T extends string>({ value, options, onChange }: CustomSelectProps<T>) {
  const [open, setOpen] = useState(false);
  const containerRef = useRef<HTMLDivElement | null>(null);
  const selected = options.find((item) => item.value === value) ?? options[0];

  useEffect(() => {
    if (!open) {
      return;
    }
    const onPointerDown = (event: MouseEvent) => {
      if (containerRef.current && !containerRef.current.contains(event.target as Node)) {
        setOpen(false);
      }
    };
    const onKeyDown = (event: KeyboardEvent) => {
      if (event.key === "Escape") {
        setOpen(false);
      }
    };
    window.addEventListener("mousedown", onPointerDown);
    window.addEventListener("keydown", onKeyDown);
    return () => {
      window.removeEventListener("mousedown", onPointerDown);
      window.removeEventListener("keydown", onKeyDown);
    };
  }, [open]);

  return (
    <div ref={containerRef} className="relative">
      <button
        type="button"
        aria-haspopup="listbox"
        aria-expanded={open}
        onClick={() => setOpen((prev) => !prev)}
        className={cn(
          "flex h-11 w-full items-center justify-between gap-3 rounded-2xl border px-4 text-[14px] font-medium text-fg transition-all",
          open ? "border-mint bg-card shadow-[0_0_0_3px_rgba(0,229,153,0.14)]" : "border-line bg-chip hover:border-mint/50",
        )}
      >
        <span className="flex min-w-0 items-center gap-2.5 truncate">
          {selected?.icon}
          <span className="truncate">{selected?.label}</span>
          {selected?.sublabel && <span className="mono text-xs text-fg2">({selected.sublabel})</span>}
        </span>
        <ChevronDown
          size={17}
          className={cn("shrink-0 text-fg2 transition-transform duration-200", open && "rotate-180 text-mint")}
        />
      </button>

      <AnimatePresence>
        {open && (
          <motion.div
            role="listbox"
            initial={{ opacity: 0, y: -6, scale: 0.98 }}
            animate={{ opacity: 1, y: 0, scale: 1 }}
            exit={{ opacity: 0, y: -6, scale: 0.98 }}
            transition={{ duration: 0.14 }}
            className="surface-dark scroll-y absolute inset-x-0 top-[calc(100%+6px)] z-50 max-h-60 overflow-y-auto rounded-2xl border border-white/12 bg-[#121918] p-1.5 text-fg shadow-2xl"
          >
            {options.map((option) => {
              const active = option.value === value;
              return (
                <button
                  key={option.value}
                  type="button"
                  role="option"
                  aria-selected={active}
                  onClick={() => {
                    onChange(option.value);
                    setOpen(false);
                  }}
                  className={cn(
                    "flex w-full items-center justify-between gap-2.5 rounded-xl border-0 px-3 py-2.5 text-start text-[13.5px] font-medium transition-colors",
                    active ? "bg-mint/15 font-semibold text-mint" : "bg-transparent text-fg hover:bg-white/8",
                  )}
                >
                  <span className="flex min-w-0 items-center gap-2.5 truncate">
                    {option.icon}
                    <span className="truncate">{option.label}</span>
                    {option.sublabel && <span className="mono text-xs text-fg2">{option.sublabel}</span>}
                  </span>
                  {active && (
                    <span className="grid h-5 w-5 shrink-0 place-items-center rounded-full bg-mint text-[#04231a]">
                      <Check size={12} strokeWidth={3} />
                    </span>
                  )}
                </button>
              );
            })}
          </motion.div>
        )}
      </AnimatePresence>
    </div>
  );
}

interface ModalProps {
  open: boolean;
  title: string;
  onClose: () => void;
  children: ReactNode;
  footer?: ReactNode;
  wide?: boolean;
}

export function Modal({ open, title, onClose, children, footer, wide = false }: ModalProps) {
  const { isMobile, t, resolvedTheme } = useApp();

  useEffect(() => {
    if (!open) {
      return;
    }
    const handler = (event: KeyboardEvent) => {
      if (event.key === "Escape") {
        onClose();
      }
    };
    window.addEventListener("keydown", handler);
    return () => window.removeEventListener("keydown", handler);
  }, [open, onClose]);

  return (
    <AnimatePresence>
      {open && (
        <motion.div
          className={cn("fixed inset-0 z-50 flex bg-black/60 backdrop-blur-sm", isMobile ? "items-end" : "items-center justify-center p-6")}
          initial={{ opacity: 0 }}
          animate={{ opacity: 1 }}
          exit={{ opacity: 0 }}
          onMouseDown={(event) => {
            if (event.target === event.currentTarget) {
              onClose();
            }
          }}
        >
          <motion.div
            role="dialog"
            aria-label={title}
            data-theme={resolvedTheme}
            className={cn(
              "surface-dark flex max-h-[88vh] flex-col border border-white/10 bg-hero text-fg shadow-2xl",
              isMobile ? "w-full rounded-t-[32px]" : cn("w-full rounded-[28px]", wide ? "max-w-2xl" : "max-w-lg"),
            )}
            style={
              isMobile
                ? {
                    paddingBottom: "max(env(safe-area-inset-bottom, 0px), var(--android-inset-bottom, 12px))",
                  }
                : undefined
            }
            initial={isMobile ? { y: "100%" } : { opacity: 0, scale: 0.96, y: 12 }}
            animate={isMobile ? { y: 0 } : { opacity: 1, scale: 1, y: 0 }}
            exit={isMobile ? { y: "100%" } : { opacity: 0, scale: 0.96 }}
            transition={{ type: "spring", damping: 30, stiffness: 340 }}
          >
            {isMobile && <div className="mx-auto mt-3 h-1 w-14 rounded-full bg-white/30" />}
            <div className="flex items-center justify-between px-6 pb-2 pt-5">
              <h2 className="m-0 text-xl font-bold">{title}</h2>
              <button
                type="button"
                aria-label={t("done")}
                onClick={onClose}
                className="grid h-9 w-9 place-items-center rounded-full border-0 bg-white/8 text-fg2 hover:text-fg"
              >
                <X size={18} />
              </button>
            </div>
            <div className="scroll-y min-h-0 flex-1 px-6 pb-4 pt-2">{children}</div>
            {footer && <div className="flex items-center justify-end gap-3 border-t border-line px-6 py-4">{footer}</div>}
          </motion.div>
        </motion.div>
      )}
    </AnimatePresence>
  );
}

interface ButtonProps {
  children: ReactNode;
  onClick?: () => void;
  variant?: "mint" | "outline" | "ghost";
  disabled?: boolean;
  className?: string;
  type?: "button" | "submit";
}

export function Button({ children, onClick, variant = "mint", disabled = false, className = "", type = "button" }: ButtonProps) {
  return (
    <button
      type={type}
      onClick={onClick}
      disabled={disabled}
      className={cn(
        "inline-flex items-center justify-center gap-2 rounded-full px-5 py-2.5 text-sm font-bold transition-all disabled:cursor-not-allowed disabled:opacity-50",
        variant === "mint" && "border-0 bg-mint text-[#06251a] hover:bg-mint-hover",
        variant === "outline" && "border border-mint bg-transparent text-mint hover:bg-mint/10",
        variant === "ghost" && "border border-line bg-transparent text-fg hover:bg-chip",
        className,
      )}
    >
      {children}
    </button>
  );
}

export function Toast() {
  const { toast, isMobile } = useApp();
  return (
    <div
      className="pointer-events-none fixed inset-x-0 z-[60] flex justify-center md:bottom-10"
      style={
        isMobile
          ? {
              bottom: "calc(max(env(safe-area-inset-bottom, 0px), var(--android-inset-bottom, 12px)) + 98px)",
            }
          : undefined
      }
    >
      <AnimatePresence>
        {toast && (
          <motion.div
            key={toast.id}
            initial={{ opacity: 0, y: 16 }}
            animate={{ opacity: 1, y: 0 }}
            exit={{ opacity: 0, y: 10 }}
            className="flex items-center gap-2 rounded-full border border-white/10 bg-[#1d2a27] px-4 py-2 text-sm font-semibold text-white shadow-xl"
          >
            <Star size={16} className={toast.starred ? "fill-star text-star" : "text-white/70"} />
            {toast.text}
          </motion.div>
        )}
      </AnimatePresence>
    </div>
  );
}