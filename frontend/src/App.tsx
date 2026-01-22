import React, { useEffect } from 'react'
import { useEditorStore } from './store/useEditorStore'
import Timeline from './components/Timeline'
import LayerPanel from './components/LayerPanel'
import PropertiesPanel from './components/PropertiesPanel'
import PreviewWindow from './components/PreviewWindow'
import AgentPanel from './components/AgentPanel'
import Toolbar from './components/Toolbar'

function App() {
  const { connect, setProjectId } = useEditorStore()

  useEffect(() => {
    connect()
    // For now, let's assume a default project
    setProjectId('default-project')
  }, [])

  return (
    <div className="flex flex-col h-screen w-screen bg-background text-white overflow-hidden">
      {/* Top Toolbar */}
      <Toolbar />
      
      {/* Main Content Area */}
      <div className="flex flex-1 min-h-0">
        <div className="flex flex-col flex-1 min-w-0">
          {/* Preview Window */}
          <div className="flex-1 bg-black flex items-center justify-center relative">
            <PreviewWindow />
          </div>
          
          {/* Timeline Area */}
          <div className="h-1/3 min-h-[300px] border-t border-timeline-border flex">
            <LayerPanel />
            <div className="flex-1 overflow-hidden relative">
              <Timeline />
            </div>
          </div>
        </div>
        
        {/* Right Sidebar */}
        <div className="w-80 border-l border-timeline-border flex flex-col">
          <PropertiesPanel />
          <AgentPanel />
        </div>
      </div>
    </div>
  )
}

export default App
