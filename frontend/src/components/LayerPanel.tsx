import React from 'react'
import { useEditorStore } from '../store/useEditorStore'
import { Eye, Lock, Volume2, Video, Sparkles, Settings2 } from 'lucide-react'

const LayerPanel: React.FC = () => {
  const { timeline } = useEditorStore()

  if (!timeline) return (
    <div className="w-64 bg-timeline-header border-r border-timeline-border p-4 text-gray-500">
      No layers
    </div>
  )

  const getIcon = (kind: string) => {
    switch (kind) {
      case 'Video': return <Video size={14} />
      case 'Audio': return <Volume2 size={14} />
      case 'Vfx': return <Sparkles size={14} />
      case 'Adjustment': return <Settings2 size={14} />
      default: return null
    }
  }

  return (
    <div className="w-64 bg-timeline-header border-r border-timeline-border flex flex-col">
      <div className="h-8 border-b border-timeline-border flex items-center px-4 text-xs font-bold text-gray-400">
        LAYERS
      </div>
      <div className="flex-1 overflow-y-auto">
        {timeline.layers.map((layer) => (
          <div 
            key={layer.id[0]} 
            className="h-24 border-b border-timeline-border p-2 flex flex-col justify-between hover:bg-white/5 transition group"
          >
            <div className="flex items-center justify-between">
              <div className="flex items-center space-x-2 truncate">
                <span className="text-gray-400 group-hover:text-primary transition-colors">
                  {getIcon(layer.kind)}
                </span>
                <span className="text-sm font-medium truncate">{layer.name}</span>
              </div>
              <div className="flex items-center space-x-1">
                <button className="p-1 hover:bg-gray-700 rounded text-gray-500 hover:text-white"><Eye size={14} /></button>
                <button className="p-1 hover:bg-gray-700 rounded text-gray-500 hover:text-white"><Lock size={14} /></button>
              </div>
            </div>
            
            <div className="text-[10px] text-gray-500 uppercase tracking-wider">
              {layer.kind} Layer
            </div>
          </div>
        ))}
      </div>
    </div>
  )
}

export default LayerPanel
