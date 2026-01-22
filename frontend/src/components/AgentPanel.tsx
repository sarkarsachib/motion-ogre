import React, { useState } from 'react'
import { Send, Sparkles, Loader2 } from 'lucide-react'
import { useEditorStore } from '../store/useEditorStore'

const AgentPanel: React.FC = () => {
  const [intent, setIntent] = useState('')
  const { agentStatus, sendCommand } = useEditorStore()

  const handleExecute = () => {
    if (!intent.trim() || agentStatus.isExecuting) return
    
    sendCommand({
      type: 'agent_execute',
      agent_type: 'DirectorAgent',
      intent: intent,
    } as any)
    
    setIntent('')
  }

  return (
    <div className="h-1/2 flex flex-col bg-surface">
      <div className="h-8 border-b border-timeline-border flex items-center px-4 text-xs font-bold text-gray-400 justify-between">
        <div className="flex items-center space-x-2">
          <Sparkles size={12} className="text-accent" />
          <span>AI ASSISTANT</span>
        </div>
        {agentStatus.isExecuting && (
          <div className="flex items-center space-x-1 text-[10px] text-accent animate-pulse">
            <Loader2 size={10} className="animate-spin" />
            <span>RUNNING</span>
          </div>
        )}
      </div>

      <div className="flex-1 overflow-y-auto p-4 space-y-4">
        {agentStatus.isExecuting ? (
          <div className="bg-background rounded-lg p-3 border border-gray-800">
            <div className="flex items-center justify-between mb-2">
              <span className="text-xs text-gray-400">Processing Intent...</span>
              <span className="text-xs text-accent">{agentStatus.progress}%</span>
            </div>
            <div className="w-full bg-gray-800 rounded-full h-1.5 mb-2">
              <div 
                className="bg-accent h-1.5 rounded-full transition-all duration-300" 
                style={{ width: `${agentStatus.progress}%` }}
              />
            </div>
            <p className="text-xs text-gray-300 italic">"{agentStatus.message}"</p>
          </div>
        ) : (
          <div className="text-center py-8">
            <Sparkles size={32} className="mx-auto text-gray-700 mb-2" />
            <p className="text-sm text-gray-500">How can I help you edit today?</p>
          </div>
        )}
      </div>

      <div className="p-4 border-t border-timeline-border">
        <div className="relative">
          <textarea
            value={intent}
            onChange={(e) => setIntent(e.target.value)}
            placeholder="e.g., 'Make the transitions more dramatic' or 'Remove the silence between clips'"
            className="w-full bg-background border border-gray-700 rounded-lg pl-3 pr-10 py-2 text-sm focus:outline-none focus:border-primary resize-none h-24"
            onKeyDown={(e) => {
              if (e.key === 'Enter' && !e.shiftKey) {
                e.preventDefault()
                handleExecute()
              }
            }}
          />
          <button 
            onClick={handleExecute}
            disabled={!intent.trim() || agentStatus.isExecuting}
            className="absolute right-2 bottom-2 p-1.5 bg-primary disabled:bg-gray-700 text-white rounded-md transition hover:bg-blue-600"
          >
            <Send size={16} />
          </button>
        </div>
        
        <div className="mt-2 flex flex-wrap gap-2">
          {['Tighten cuts', 'Auto color', 'Sync to beat'].map(suggestion => (
            <button 
              key={suggestion}
              onClick={() => setIntent(suggestion)}
              className="text-[10px] bg-gray-800 hover:bg-gray-700 px-2 py-1 rounded transition text-gray-400"
            >
              {suggestion}
            </button>
          ))}
        </div>
      </div>
    </div>
  )
}

export default AgentPanel
