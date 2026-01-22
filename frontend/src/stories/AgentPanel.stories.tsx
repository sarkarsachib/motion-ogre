import type { Meta, StoryObj } from '@storybook/react';
import AgentPanel from '../components/AgentPanel';
import React from 'react';

const meta: Meta<typeof AgentPanel> = {
  title: 'Components/AgentPanel',
  component: AgentPanel,
  decorators: [
    (Story) => (
      <div className="bg-background text-white h-[600px] w-80 border border-gray-800">
        <Story />
      </div>
    ),
  ],
};

export default meta;
type Story = StoryObj<typeof AgentPanel>;

export const Default: Story = {};
