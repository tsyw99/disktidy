import { MainLayout } from './components/layout';
import { NavigationDock, TitleBar } from './components/common';
import {
  SystemPage,
  ScanPage,
  CleanPage,
  FileAnalysisPage,
  ToolsPage,
  SettingsPage
} from './pages';
import { useUIStore, type UIState } from './stores';

function App() {
  const currentPage = useUIStore((state: UIState) => state.currentPage);

  const renderPage = () => {
    switch (currentPage) {
      case 'system':
        return <SystemPage />;
      case 'scan':
        return <ScanPage />;
      case 'clean':
        return <CleanPage />;
      case 'analyze':
        return <FileAnalysisPage />;
      case 'tools':
        return <ToolsPage />;
      case 'settings':
        return <SettingsPage />;
      default:
        return <SystemPage />;
    }
  };

  return (
    <div className="h-screen w-screen flex flex-col overflow-hidden">
      <TitleBar />
      <MainLayout>
        {renderPage()}
        <NavigationDock />
      </MainLayout>
    </div>
  );
}

export default App;
