import React from 'react'
import { Play, Pause, Square, SkipBack, SkipForward, Undo, Redo, Save, FolderOpen, Plus } from 'lucide-react'
import { useEditorStore } from '../store/useEditorStore'

const Toolbar: React.FC = () => {
  const { isPlaying, sendCommand } = useEditorStore()

  const togglePlay = () => {
    if (isPlaying) {
      sendCommand({ type: 'playhead_pause' })
    } else {
      sendCommand({ type: 'playhead_play' })
    }
  }

  return (
    <div className="h-12 border-b border-timeline-border bg-surface flex items-center px-4 justify-between">
      <div className="flex items-center space-x-4">
        <div className="flex items-center space-x-2 mr-4">
          <span className="font-bold text-primary">MOTION OGRE</span>
        </div>
        
        <div className="flex items-center space-x-1 border-r border-gray-700 pr-4">
          <button className="p-1.5 hover:bg-gray-700 rounded transition" title="New Project"><Plus size={18} /></button>
          <button className="p-1.5 hover:bg-gray-700 rounded transition" title="Open Project"><FolderOpen size={18} /></button>
          <button className="p-1.5 hover:bg-gray-700 rounded transition" title="Save Project"><Save size={18} /></button>
        </div>

        <div className="flex items-center space-x-1">
          <button className="p-1.5 hover:bg-gray-700 rounded transition" title="Undo"><Undo size={18} /></button>
          <button className="p-1.5 hover:bg-gray-700 rounded transition" title="Redo"><Redo size={18} /></button>
        </div>
      </div>

      <div className="flex items-center space-x-2">
        <button className="p-1.5 hover:bg-gray-700 rounded transition"><SkipBack size={18} /></button>
        <button 
          onClick={togglePlay}
          className="p-2 bg-primary hover:bg-blue-600 rounded-full transition shadow-lg"
        >
          {isPlaying ? <Pause size={20} fill="currentColor" /> : <Play size={20} fill="currentColor" className="ml-0.5" />}
        </button>
        <button className="p-1.5 hover:bg-gray-700 rounded transition"><Square size={18} fill="currentColor" /></button>
        <button className="p-1.5 hover:bg-gray-700 rounded transition"><SkipForward size={18} /></button>
      </div>

      <div className="w-48 flex justify-end">
        <button className="px-4 py-1.5 bg-secondary hover:bg-green-600 rounded text-sm font-medium transition">
          Export
        </button>
      </div>
    </div>
  )
}

export default Toolbar
