import React from 'react'
import { useEditorStore } from '../store/useEditorStore'

const PreviewWindow: React.FC = () => {
  const { playheadTime } = useEditorStore()

  const formatTime = (time: { numerator: number, denominator: number }) => {
    const totalSeconds = time.numerator / time.denominator
    const minutes = Math.floor(totalSeconds / 60)
    const seconds = Math.floor(totalSeconds % 60)
    const frames = Math.floor((totalSeconds % 1) * 30) // Assuming 30fps for display
    return `${minutes.toString().padStart(2, '0')}:${seconds.toString().padStart(2, '0')}:${frames.toString().padStart(2, '0')}`
  }

  return (
    <div className="w-full h-full flex flex-col items-center justify-center bg-black">
      <div className="aspect-video w-full max-w-4xl bg-gray-900 shadow-2xl flex items-center justify-center border border-gray-800 relative">
         <div className="text-gray-600 font-mono text-xl">
           VIDEO PREVIEW AREA
         </div>
         
         {/* Resolution badge */}
         <div className="absolute top-4 right-4 bg-black/50 px-2 py-1 rounded text-xs text-gray-400">
           1080p | 30fps
         </div>
      </div>
      
      <div className="mt-4 font-mono text-2xl text-primary tabular-nums">
        {formatTime(playheadTime)}
      </div>
    </div>
  )
}

export default PreviewWindow
