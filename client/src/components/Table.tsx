import { HTMLTable } from "@blueprintjs/core";
import { ReactElement } from "react";

export interface TableProps {
  columnNames: string[],
  children: ReactElement[] | undefined | boolean
}

export function Table({ columnNames, children }: TableProps) {
  if (!children) {
    return <></>;
  }

  return (
    <HTMLTable compact={true} striped={true}>
      <thead>
        <tr>
          {columnNames.map(c => <th key={c}>{c}</th>)}
        </tr>
      </thead>
      <tbody>
        {children}
      </tbody>
    </HTMLTable>
  );
}