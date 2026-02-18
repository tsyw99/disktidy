import { MainLayout } from './components/layout';
import { NavigationDock } from './components/common';
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
    <MainLayout>
      {renderPage()}
      <NavigationDock />
    </MainLayout>
  );
}

export default App;
