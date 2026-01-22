import { create } from 'zustand'
import { Timeline, Track, Layer, Clip } from '../types/timeline'
import { Event, Command } from '../types/commands'

interface EditorState {
  timeline: Timeline | null
  projectId: string | null
  playheadTime: { numerator: number; denominator: number }
  isPlaying: boolean
  selectedClipId: number | null
  zoomLevel: number
  agentStatus: {
    isExecuting: boolean
    progress: number
    message: string
  }
  
  // Actions
  setTimeline: (timeline: Timeline) => void
  setProjectId: (id: string) => void
  setPlayheadTime: (time: { numerator: number; denominator: number }) => void
  setIsPlaying: (isPlaying: boolean) => void
  setSelectedClipId: (clipId: number | null) => void
  setZoomLevel: (zoom: number) => void
  setAgentStatus: (status: Partial<EditorState['agentStatus']>) => void
  
  // WebSocket
  ws: WebSocket | null
  connect: () => void
  sendCommand: (command: Omit<Command, 'id' | 'timestamp'>) => void
}

export const useEditorStore = create<EditorState>((set, get) => ({
  timeline: null,
  projectId: null,
  playheadTime: { numerator: 0, denominator: 1 },
  isPlaying: false,
  selectedClipId: null,
  zoomLevel: 10,
  agentStatus: {
    isExecuting: false,
    progress: 0,
    message: '',
  },
  ws: null,

  setTimeline: (timeline) => set({ timeline }),
  setProjectId: (projectId) => {
    set({ projectId });
    // Initialize with mock data if timeline is null
    if (!get().timeline) {
      const mockTimeline: Timeline = {
        frame_rate: { fps: { numerator: 30, denominator: 1 } },
        layers: [
          {
            id: { 0: 1 },
            name: 'Video Layer 1',
            kind: 'Video',
            z_order: 0,
            tracks: [
              {
                id: { 0: 101 },
                name: 'Main Track',
                clips: Array.from({ length: 12 }).map((_, i) => ({
                  id: { 0: 1000 + i },
                  name: `Clip ${i + 1}`,
                  enabled: true,
                  range: {
                    start: { numerator: i * 5, denominator: 1 },
                    duration: { numerator: 4, denominator: 1 }
                  },
                  source: { PlaceholderMedia: { media_id: `m${i}` } }
                }))
              }
            ]
          },
          {
            id: { 0: 2 },
            name: 'Audio Layer 1',
            kind: 'Audio',
            z_order: 1,
            tracks: [
              {
                id: { 0: 201 },
                name: 'Audio Track',
                clips: [
                  {
                    id: { 0: 2000 },
                    name: 'Background Music',
                    enabled: true,
                    range: {
                      start: { numerator: 0, denominator: 1 },
                      duration: { numerator: 60, denominator: 1 }
                    },
                    source: { PlaceholderMedia: { media_id: 'bgm' } }
                  }
                ]
              }
            ]
          }
        ],
        next_id: 10000
      };
      set({ timeline: mockTimeline });
    }
  },
  setPlayheadTime: (playheadTime) => set({ playheadTime }),
  setIsPlaying: (isPlaying) => set({ isPlaying }),
  setSelectedClipId: (selectedClipId) => set({ selectedClipId }),
  setZoomLevel: (zoomLevel) => set({ zoomLevel }),
  setAgentStatus: (status) => set((state) => ({ 
    agentStatus: { ...state.agentStatus, ...status } 
  })),

  connect: () => {
    const protocol = window.location.protocol === 'https:' ? 'wss:' : 'ws:';
    // Use the dev server proxy if in development, otherwise direct
    const host = window.location.hostname === 'localhost' ? 'localhost:3001' : window.location.host;
    const ws = new WebSocket(`${protocol}//${host}`);

    ws.onmessage = (event) => {
      const data = JSON.parse(event.data) as Event | { type: 'response', data: any };
      
      if ('type' in data) {
        switch (data.type) {
          case 'timeline_updated':
            set({ timeline: data.timeline });
            break;
          case 'playhead_moved':
            set({ playheadTime: data.time, isPlaying: data.is_playing });
            break;
          case 'playhead_playing':
            set({ isPlaying: true });
            break;
          case 'playhead_paused':
            set({ isPlaying: false, playheadTime: data.time });
            break;
          case 'agent_started':
            set({ agentStatus: { isExecuting: true, progress: 0, message: 'Agent started...' } });
            break;
          case 'agent_progress':
            set({ agentStatus: { isExecuting: true, progress: data.progress, message: data.message } });
            break;
          case 'agent_completed':
            set({ agentStatus: { isExecuting: false, progress: 100, message: 'Completed' } });
            break;
        }
      }
    };

    ws.onopen = () => {
      console.log('WebSocket connected');
    };

    ws.onclose = () => {
      console.log('WebSocket disconnected');
      setTimeout(() => get().connect(), 3000);
    };

    set({ ws });
  },

  sendCommand: (commandData) => {
    const { ws, projectId } = get();
    if (!ws || ws.readyState !== WebSocket.OPEN) return;

    const command = {
      ...commandData,
      id: crypto.randomUUID(),
      timestamp: Date.now(),
      project_id: projectId,
    };

    ws.send(JSON.stringify(command));
  }
}))
