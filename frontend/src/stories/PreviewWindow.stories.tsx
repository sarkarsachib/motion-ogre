import type { Meta, StoryObj } from '@storybook/react';
import PreviewWindow from '../components/PreviewWindow';
import React from 'react';

const meta: Meta<typeof PreviewWindow> = {
  title: 'Components/PreviewWindow',
  component: PreviewWindow,
  decorators: [
    (Story) => (
      <div className="bg-background text-white h-[400px] w-full p-4">
        <Story />
      </div>
    ),
  ],
};

export default meta;
type Story = StoryObj<typeof PreviewWindow>;

export const Default: Story = {};
