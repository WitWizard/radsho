import { useApp } from "../state/AppContext";
import { Modal } from "./ui";
import { OptionCards } from "./OptionCards";

export function OptionsDrawer() {
  const { t, optionsOpen, setOptionsOpen } = useApp();
  return (
    <Modal open={optionsOpen} title={t("vpnOptions")} onClose={() => setOptionsOpen(false)}>
      <OptionCards variant="drawer" />
    </Modal>
  );
}
