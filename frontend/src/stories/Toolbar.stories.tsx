import type { Meta, StoryObj } from '@storybook/react';
import Toolbar from '../components/Toolbar';
import React from 'react';

const meta: Meta<typeof Toolbar> = {
  title: 'Components/Toolbar',
  component: Toolbar,
  parameters: {
    layout: 'fullscreen',
  },
  decorators: [
    (Story) => (
      <div className="bg-background text-white h-12">
        <Story />
      </div>
    ),
  ],
};

export default meta;
type Story = StoryObj<typeof Toolbar>;

export const Default: Story = {};
