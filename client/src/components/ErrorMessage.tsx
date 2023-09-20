import { Callout } from "@blueprintjs/core"
import { css } from "@emotion/react"

export interface Props {
  message: string
}

export function ErrorMessage({ message } : Props) {
  return (
    <div css={css`text-align: left`}>
      <Callout intent="warning" title="Unable to load">
        Error: {message}
      </Callout>
    </div>
  )
}