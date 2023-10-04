import { HTMLTable } from "@blueprintjs/core";
import { css } from "@emotion/react";
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
        <tr css={css`white-space: nowrap;`}>
          {columnNames.map((c, i) => <th key={i}>{c}</th>)}
        </tr>
      </thead>
      <tbody>
        {children}
      </tbody>
    </HTMLTable>
  );
}