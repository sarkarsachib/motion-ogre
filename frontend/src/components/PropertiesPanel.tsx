import React from 'react'
import { useEditorStore } from '../store/useEditorStore'

const PropertiesPanel: React.FC = () => {
  const { timeline, selectedClipId } = useEditorStore()

  const findClip = () => {
    if (!timeline || selectedClipId === null) return null
    for (const layer of timeline.layers) {
      for (const track of layer.tracks) {
        const clip = track.clips.find(c => c.id[0] === selectedClipId)
        if (clip) return clip
      }
    }
    return null
  }

  const clip = findClip()

  return (
    <div className="flex-1 bg-surface border-b border-timeline-border flex flex-col">
      <div className="h-8 border-b border-timeline-border flex items-center px-4 text-xs font-bold text-gray-400">
        PROPERTIES
      </div>
      
      {clip ? (
        <div className="p-4 space-y-4">
          <div>
            <label className="text-[10px] text-gray-500 uppercase block mb-1">Name</label>
            <input 
              type="text" 
              value={clip.name} 
              readOnly
              className="w-full bg-background border border-gray-700 rounded px-2 py-1 text-sm focus:outline-none focus:border-primary"
            />
          </div>
          
          <div className="grid grid-cols-2 gap-4">
            <div>
              <label className="text-[10px] text-gray-500 uppercase block mb-1">Start</label>
              <div className="bg-background border border-gray-700 rounded px-2 py-1 text-sm text-gray-400">
                {(clip.range.start.numerator / clip.range.start.denominator).toFixed(2)}s
              </div>
            </div>
            <div>
              <label className="text-[10px] text-gray-500 uppercase block mb-1">Duration</label>
              <div className="bg-background border border-gray-700 rounded px-2 py-1 text-sm text-gray-400">
                {(clip.range.duration.numerator / clip.range.duration.denominator).toFixed(2)}s
              </div>
            </div>
          </div>

          <div className="pt-4 border-t border-gray-800">
            <h4 className="text-xs font-bold text-gray-400 mb-2">TRANSFORM</h4>
            <div className="space-y-2">
              <PropertyRow label="Opacity" value="100%" />
              <PropertyRow label="Scale" value="100%" />
              <PropertyRow label="Position X" value="0" />
              <PropertyRow label="Position Y" value="0" />
            </div>
          </div>
        </div>
      ) : (
        <div className="flex-1 flex items-center justify-center text-gray-600 text-sm">
          Select a clip to view properties
        </div>
      )}
    </div>
  )
}

const PropertyRow = ({ label, value }: { label: string, value: string }) => (
  <div className="flex items-center justify-between text-sm">
    <span className="text-gray-400">{label}</span>
    <span className="text-primary font-mono">{value}</span>
  </div>
)

export default PropertiesPanel
