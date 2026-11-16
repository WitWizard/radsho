import { useApp } from "../state/AppContext";
import { Page, Card } from "../components/Page";
import { SpeedPanel } from "../components/AdvancedModals";

export function SpeedView() {
  const { t } = useApp();
  return (
    <Page title={t("speedTitle")}>
      <div className="mx-auto max-w-3xl">
        <Card>
          <SpeedPanel />
        </Card>
      </div>
    </Page>
  );
}