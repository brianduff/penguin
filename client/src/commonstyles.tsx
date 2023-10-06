import { css } from "@emotion/react";

export const gridStyle = css`
  display: grid;
  grid-gap: 18px;
  grid-template-columns: 1fr;
  width: 100%;
  @media (min-width: 420px) {
    grid-template-columns: 1fr 1fr;
  }
`;
