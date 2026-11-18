import type { ReactNode } from "react";
import { cn } from "../utils/cn";
import { useApp } from "../state/AppContext";

interface PageProps {
  title: string;
  children: ReactNode;
  actions?: ReactNode;
  fill?: boolean;
}

export function Page({ title, children, actions, fill = false }: PageProps) {
  const { isMobile } = useApp();
  return (
    <div className={cn("flex h-full min-h-0 flex-col", isMobile ? "px-5 pt-4" : "pe-1")}>
      <div className="mb-4 flex shrink-0 items-center justify-between gap-3">
        <h1 className={cn("m-0 font-semibold tracking-tight text-fg", isMobile ? "text-[30px]" : "text-[40px]")}>{title}</h1>
        {actions && <div className="flex items-center gap-2">{actions}</div>}
      </div>
      <div className={cn("min-h-0 flex-1 pb-4", fill ? "flex flex-col" : "scroll-y")}>
        {children}
      </div>
    </div>
  );
}

interface CardProps {
  title?: string;
  children: ReactNode;
  className?: string;
}

export function Card({ title, children, className = "" }: CardProps) {
  return (
    <section className={cn("rounded-[28px] border border-line bg-card p-5 shadow-[var(--shadow)]", className)}>
      {title && <h2 className="m-0 mb-2 text-[18px] font-semibold text-fg">{title}</h2>}
      {children}
    </section>
  );
}