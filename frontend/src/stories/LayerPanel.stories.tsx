import type { Meta, StoryObj } from '@storybook/react';
import LayerPanel from '../components/LayerPanel';
import React from 'react';

const meta: Meta<typeof LayerPanel> = {
  title: 'Components/LayerPanel',
  component: LayerPanel,
  decorators: [
    (Story) => (
      <div className="bg-background text-white h-[400px] w-64 border border-gray-800">
        <Story />
      </div>
    ),
  ],
};

export default meta;
type Story = StoryObj<typeof LayerPanel>;

export const Default: Story = {};
