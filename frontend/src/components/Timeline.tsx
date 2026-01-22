import React, { useRef, useEffect } from 'react'
import { useEditorStore } from '../store/useEditorStore'
import { Clip as ClipType } from '../types/timeline'
import { clsx } from 'clsx'

const Timeline: React.FC = () => {
  const { timeline, playheadTime, zoomLevel, setPlayheadTime, sendCommand, selectedClipId, setSelectedClipId } = useEditorStore()
  const containerRef = useRef<HTMLDivElement>(null)

  const pixelsPerSecond = zoomLevel * 10
  
  const handleTimelineClick = (e: React.MouseEvent) => {
    if (!containerRef.current) return
    const rect = containerRef.current.getBoundingClientRect()
    const x = e.clientX - rect.left + containerRef.current.scrollLeft
    const timeInSeconds = x / pixelsPerSecond
    
    const newTime = {
      numerator: Math.round(timeInSeconds * 1000),
      denominator: 1000
    }
    
    setPlayheadTime(newTime)
    sendCommand({
      type: 'playhead_move',
      time: newTime
    } as any)
  }

  if (!timeline) return <div className="p-8 text-gray-500">Loading timeline...</div>

  return (
    <div 
      ref={containerRef}
      className="h-full bg-timeline-bg overflow-x-auto overflow-y-hidden relative select-none"
      onClick={handleTimelineClick}
    >
      {/* Time Ruler */}
      <div className="h-8 border-b border-timeline-border sticky top-0 bg-timeline-header z-20 flex items-end">
        {Array.from({ length: 100 }).map((_, i) => (
          <div 
            key={i} 
            className="border-l border-gray-600 h-2 flex-shrink-0 relative"
            style={{ width: pixelsPerSecond }}
          >
            <span className="absolute -top-6 left-1 text-[10px] text-gray-400 font-mono">
              {i}s
            </span>
          </div>
        ))}
      </div>

      {/* Layers Content */}
      <div className="relative">
        {timeline.layers.map((layer) => (
          <div 
            key={layer.id[0]} 
            className="h-24 border-b border-timeline-border relative group"
          >
            {layer.tracks.map((track) => (
              <div key={track.id[0]} className="h-full w-full relative">
                {track.clips.map((clip) => (
                  <Clip 
                    key={clip.id[0]} 
                    clip={clip} 
                    pixelsPerSecond={pixelsPerSecond}
                    isSelected={selectedClipId === clip.id[0]}
                    onSelect={() => setSelectedClipId(clip.id[0])}
                    onMove={(newStart) => {
                      sendCommand({
                        type: 'clip_update',
                        data: {
                          layer_id: layer.id,
                          track_id: track.id,
                          clip_id: clip.id,
                          range: {
                            ...clip.range,
                            start: { numerator: Math.round(newStart * 1000), denominator: 1000 }
                          }
                        }
                      } as any)
                    }}
                  />
                ))}
              </div>
            ))}
          </div>
        ))}
      </div>

      {/* Playhead */}
      <div 
        className="absolute top-0 bottom-0 w-px bg-red-500 z-30 pointer-events-none"
        style={{ 
          left: (playheadTime.numerator / playheadTime.denominator) * pixelsPerSecond,
          transition: 'left 0.1s linear'
        }}
      >
        <div className="w-3 h-3 bg-red-500 rounded-full -ml-1.5 mt-6 shadow-sm shadow-black" />
      </div>
    </div>
  )
}

interface ClipProps {
  clip: ClipType
  pixelsPerSecond: number
  isSelected: boolean
  onSelect: () => void
  onMove: (newStart: number) => void
}

const Clip: React.FC<ClipProps> = ({ clip, pixelsPerSecond, isSelected, onSelect, onMove }) => {
  const startSeconds = clip.range.start.numerator / clip.range.start.denominator
  const durationSeconds = clip.range.duration.numerator / clip.range.duration.denominator
  const [isDragging, setIsDragging] = React.useState(false)
  const [dragStart, setDragStart] = React.useState(0)
  const [dragOffset, setDragOffset] = React.useState(0)

  const handleMouseDown = (e: React.MouseEvent) => {
    e.stopPropagation()
    onSelect()
    setIsDragging(true)
    setDragStart(e.clientX)
  }

  React.useEffect(() => {
    if (!isDragging) return

    const handleMouseMove = (e: MouseEvent) => {
      const deltaX = e.clientX - dragStart
      setDragOffset(deltaX)
    }

    const handleMouseUp = () => {
      setIsDragging(false)
      const finalDeltaSeconds = dragOffset / pixelsPerSecond
      onMove(startSeconds + finalDeltaSeconds)
      setDragOffset(0)
    }

    window.addEventListener('mousemove', handleMouseMove)
    window.addEventListener('mouseup', handleMouseUp)
    return () => {
      window.removeEventListener('mousemove', handleMouseMove)
      window.removeEventListener('mouseup', handleMouseUp)
    }
  }, [isDragging, dragStart, dragOffset, pixelsPerSecond, onMove, startSeconds])

  return (
    <div 
      onMouseDown={handleMouseDown}
      className={clsx(
        "absolute top-2 h-20 rounded-md border shadow-lg cursor-grab active:cursor-grabbing overflow-hidden transition-shadow",
        isSelected ? "border-primary bg-primary/40 ring-1 ring-primary z-10" : "border-gray-600 bg-gray-700 hover:bg-gray-600 z-0",
        isDragging && "opacity-75 shadow-2xl scale-[1.02]"
      )}
      style={{
        left: (startSeconds * pixelsPerSecond) + (isDragging ? dragOffset : 0),
        width: durationSeconds * pixelsPerSecond,
      }}
    >
      <div className="px-2 py-1 text-xs font-medium truncate">
        {clip.name}
      </div>
      
      {/* Trimming handles */}
      <div className="absolute left-0 top-0 bottom-0 w-1.5 hover:bg-white/30 cursor-ew-resize" />
      <div className="absolute right-0 top-0 bottom-0 w-1.5 hover:bg-white/30 cursor-ew-resize" />
      
      {/* Background decoration */}
      <div className="absolute bottom-0 left-0 right-0 h-1 bg-black/20" />
    </div>
  )
}

export default Timeline
