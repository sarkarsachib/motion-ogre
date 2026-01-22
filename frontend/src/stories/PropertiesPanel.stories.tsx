import type { Meta, StoryObj } from '@storybook/react';
import PropertiesPanel from '../components/PropertiesPanel';
import React from 'react';

const meta: Meta<typeof PropertiesPanel> = {
  title: 'Components/PropertiesPanel',
  component: PropertiesPanel,
  decorators: [
    (Story) => (
      <div className="bg-background text-white h-[400px] w-80 border border-gray-800">
        <Story />
      </div>
    ),
  ],
};

export default meta;
type Story = StoryObj<typeof PropertiesPanel>;

export const Default: Story = {};
