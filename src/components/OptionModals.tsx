import { useState } from "react";
import { ArrowDown, ArrowUp, Check } from "lucide-react";
import { cn } from "../utils/cn";
import { useApp } from "../state/AppContext";
import { ALL_TRANSPORTS, PRESETS, SHUFFLE_INTERVALS, SHUFFLE_REGIONS, SPLIT_APPS } from "../lib/config";
import { presetLabelKey, transportLabelKey } from "../i18n/translations";
import type { PresetId, TransportId } from "../types/vpn";
import { Button, Modal } from "./ui";
import { Field, SettingRow, inputClass } from "./SettingRow";
import { Flag } from "./Flag";

function chipClass(active: boolean): string {
  return cn(
    "inline-flex items-center gap-2 rounded-full border px-3 py-1.5 text-[13px] font-semibold transition-colors",
    active ? "border-mint bg-mint/15 text-mint" : "border-white/12 bg-transparent text-fg2 hover:text-fg",
  );
}

function ShuffleForm() {
  const { t, settings, patchSettings } = useApp();
  const shuffle = settings.shuffle;

  const toggleRegion = (region: string) => {
    const has = shuffle.regions.includes(region);
    if (has && shuffle.regions.length <= 1) {
      return;
    }
    const regions = has ? shuffle.regions.filter((item) => item !== region) : [...shuffle.regions, region];
    patchSettings({ shuffle: { ...shuffle, regions } });
  };

  return (
    <div>
      <p className="m-0 mb-2 text-[14px] leading-snug text-fg2">{t("shuffleHint")}</p>
      {!settings.psiphon && (
        <p className="m-0 mb-2 rounded-2xl border border-amber-fast/35 bg-amber-fast/12 px-4 py-2 text-[13px] text-amber-fast">{t("shuffleNeedsPsiphon")}</p>
      )}
      <SettingRow title={t("shuffleTitle")} checked={shuffle.enabled} onChange={(enabled) => patchSettings({ shuffle: { ...shuffle, enabled } })} />
      <div className="py-2">
        <div className="mb-2 text-[13px] font-semibold text-fg2">{t("shuffleInterval")}</div>
        <div className="flex flex-wrap gap-2">
          {SHUFFLE_INTERVALS.map((minutes) => (
            <button
              key={minutes}
              type="button"
              onClick={() => patchSettings({ shuffle: { ...shuffle, intervalMin: minutes } })}
              className={chipClass(shuffle.intervalMin === minutes)}
            >
              <span className="mono">{minutes}</span> {t("minutes")}
            </button>
          ))}
        </div>
      </div>
      <div className="py-2">
        <div className="mb-2 text-[13px] font-semibold text-fg2">{t("shuffleRegions")}</div>
        <div className="flex flex-wrap gap-2">
          {SHUFFLE_REGIONS.map((region) => (
            <button key={region} type="button" onClick={() => toggleRegion(region)} className={chipClass(shuffle.regions.includes(region))}>
              <Flag code={region} size={20} />
              <span className="mono">{region}</span>
            </button>
          ))}
        </div>
      </div>
    </div>
  );
}

function parseLines(text: string): string[] {
  return text
    .split(/\r?\n/)
    .map((line) => line.trim())
    .filter((line) => line.length > 0);
}

function PathsForm() {
  const { t, settings, patchSettings } = useApp();
  const routing = settings.routing;
  const [direct, setDirect] = useState(routing.direct.join("\n"));
  const [block, setBlock] = useState(routing.block.join("\n"));

  const toggleApp = (name: string) => {
    const apps = routing.apps.includes(name) ? routing.apps.filter((item) => item !== name) : [...routing.apps, name];
    patchSettings({ routing: { ...routing, apps } });
  };

  return (
    <div>
      <SettingRow title={t("pathsEnable")} checked={routing.enabled} onChange={(enabled) => patchSettings({ routing: { ...routing, enabled } })} />
      <SettingRow
        title={t("pathsDomestic")}
        hint={t("pathsDomesticHint")}
        checked={routing.domesticBypass}
        onChange={(domesticBypass) => patchSettings({ routing: { ...routing, domesticBypass } })}
      />
      <Field label={t("pathsDirect")}>
        <textarea
          rows={3}
          value={direct}
          placeholder={`${t("pathsHint")}\nexample.com`}
          onChange={(event) => {
            setDirect(event.target.value);
            patchSettings({ routing: { ...routing, direct: parseLines(event.target.value) } });
          }}
          className={cn(inputClass, "mono resize-none")}
        />
      </Field>
      <Field label={t("pathsBlock")}>
        <textarea
          rows={3}
          value={block}
          placeholder={`${t("pathsHint")}\nads.example.net`}
          onChange={(event) => {
            setBlock(event.target.value);
            patchSettings({ routing: { ...routing, block: parseLines(event.target.value) } });
          }}
          className={cn(inputClass, "mono resize-none")}
        />
      </Field>
      <div className="py-2">
        <div className="mb-2 text-[13px] font-semibold text-fg2">{t("pathsApps")}</div>
        <div className="flex flex-wrap gap-2">
          {SPLIT_APPS.map((name) => (
            <button key={name} type="button" onClick={() => toggleApp(name)} className={chipClass(routing.apps.includes(name))}>
              {routing.apps.includes(name) && <Check size={14} />}
              {name}
            </button>
          ))}
        </div>
      </div>
    </div>
  );
}

function ProtocolForm() {
  const { t, settings, patchSettings } = useApp();
  const protocol = settings.protocol;
  const chain = protocol.chain;
  const ordered: TransportId[] = [...chain, ...ALL_TRANSPORTS.filter((id) => !chain.includes(id))];

  const applyPreset = (id: PresetId) => {
    const preset = PRESETS[id];
    patchSettings({
      protocol: { ...protocol, preset: id, chain: [...preset.chain], fragment: preset.fragment },
      scanMode: preset.scanMode,
      psiphon: preset.psiphon ?? settings.psiphon,
      sanctionCheck: preset.sanctionCheck ?? settings.sanctionCheck,
    });
  };

  const toggleTransport = (id: TransportId) => {
    if (chain.includes(id)) {
      if (chain.length <= 1) {
        return;
      }
      patchSettings({ protocol: { ...protocol, chain: chain.filter((item) => item !== id) } });
    } else {
      patchSettings({ protocol: { ...protocol, chain: [...chain, id] } });
    }
  };

  const move = (id: TransportId, delta: number) => {
    const index = chain.indexOf(id);
    const target = index + delta;
    if (index < 0 || target < 0 || target >= chain.length) {
      return;
    }
    const next = [...chain];
    [next[index], next[target]] = [next[target], next[index]];
    patchSettings({ protocol: { ...protocol, chain: next } });
  };

  return (
    <div>
      <SettingRow title={t("protoAuto")} checked={protocol.auto} onChange={(auto) => patchSettings({ protocol: { ...protocol, auto } })} />
      <div className="py-2">
        <div className="mb-2 text-[13px] font-semibold text-fg2">{t("protoPresets")}</div>
        <div className="flex flex-wrap gap-2">
          {(Object.keys(PRESETS) as PresetId[]).map((id) => (
            <button key={id} type="button" onClick={() => applyPreset(id)} className={chipClass(protocol.preset === id)}>
              {t(presetLabelKey[id])}
            </button>
          ))}
        </div>
      </div>
      <SettingRow title={t("protoPsiphon")} hint={t("protoPsiphonHint")} checked={settings.psiphon} onChange={(psiphon) => patchSettings({ psiphon })} />
      <SettingRow title={t("protoSanction")} hint={t("protoSanctionHint")} checked={settings.sanctionCheck} onChange={(sanctionCheck) => patchSettings({ sanctionCheck })} />
      <SettingRow
        title={t("protoFragment")}
        checked={protocol.fragment}
        onChange={(fragment) => patchSettings({ protocol: { ...protocol, fragment } })}
      />
      <div className="py-2">
        <div className="mb-2 text-[13px] font-semibold text-fg2">{t("protoChain")}</div>
        <ul className="m-0 flex list-none flex-col gap-2 p-0">
          {ordered.map((id) => {
            const index = chain.indexOf(id);
            const enabled = index >= 0;
            return (
              <li key={id} className={cn("flex items-center gap-3 rounded-2xl border px-3 py-2", enabled ? "border-white/12 bg-white/4" : "border-white/6 opacity-60")}>
                <button
                  type="button"
                  role="checkbox"
                  aria-checked={enabled}
                  aria-label={t(transportLabelKey[id])}
                  onClick={() => toggleTransport(id)}
                  className={cn("grid h-6 w-6 shrink-0 place-items-center rounded-full border", enabled ? "border-mint bg-mint text-[#04231a]" : "border-white/25 bg-transparent text-transparent")}
                >
                  <Check size={14} strokeWidth={3} />
                </button>
                <span className="mono w-5 text-center text-xs text-fg2">{enabled ? index + 1 : "–"}</span>
                <span className="min-w-0 flex-1 truncate text-[14px] font-medium">{t(transportLabelKey[id])}</span>
                {enabled && (
                  <span className="flex gap-1">
                    <button type="button" aria-label="up" onClick={() => move(id, -1)} className="grid h-7 w-7 place-items-center rounded-full border-0 bg-white/8 text-fg2 hover:text-fg">
                      <ArrowUp size={14} />
                    </button>
                    <button type="button" aria-label="down" onClick={() => move(id, 1)} className="grid h-7 w-7 place-items-center rounded-full border-0 bg-white/8 text-fg2 hover:text-fg">
                      <ArrowDown size={14} />
                    </button>
                  </span>
                )}
              </li>
            );
          })}
        </ul>
      </div>
    </div>
  );
}

export function OptionModals() {
  const { t, modal, setModal } = useApp();
  const close = () => setModal(null);
  const footer = <Button onClick={close}>{t("done")}</Button>;
  return (
    <>
      <Modal open={modal === "shuffle"} title={t("shuffleTitle")} onClose={close} footer={footer}>
        <ShuffleForm />
      </Modal>
      <Modal open={modal === "paths"} title={t("multiTitle")} onClose={close} footer={footer}>
        <PathsForm />
      </Modal>
      <Modal open={modal === "protocol"} title={t("protocolTitle")} onClose={close} footer={footer} wide>
        <ProtocolForm />
      </Modal>
    </>
  );
}
