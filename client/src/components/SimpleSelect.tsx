
import { Button, MenuItem } from '@blueprintjs/core';
import { Select, ItemRenderer } from '@blueprintjs/select';
import { useState } from 'react';

interface Props<T> {
  items: T[]
  setSelected: (value: T) => void
  render: (value: T) => string
}

export function SimpleSelect<T>({ items, setSelected, render }: Props<T>) {
  const [selectedText, setSelectedText] = useState<string>("Select a time");

  const renderer: ItemRenderer<T> = (value, { handleClick, handleFocus, modifiers }) => {
    if (!modifiers.matchesPredicate) {
        return null;
    }
    return (
        <MenuItem
            active={modifiers.active}
            disabled={modifiers.disabled}
            key={render(value)}
            onClick={handleClick}
            onFocus={handleFocus}
            roleStructure="listoption"
            text={render(value)}
        />
    );
};


  return (
    <Select<T>
        fill={true}
        filterable={false}
        popoverProps={{ minimal: true }}
        items={items}
        itemRenderer={renderer}
        onItemSelect={(item: T) => { setSelectedText(render(item)); setSelected(item); }}
    >
      <Button text={selectedText} rightIcon="double-caret-vertical" placeholder="Select a time" />
    </Select>
  )
}