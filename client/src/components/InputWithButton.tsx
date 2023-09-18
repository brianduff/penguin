import { Button, Callout, InputGroup, Popover } from "@blueprintjs/core";
import { chain, onKey } from "../events";

export interface InputWithButtonProps {
  submit: () => Promise<void>,
  value: string,
  onValueUpdated: (newValue: string) => void,
  errorMessage: string,
  setErrorMessage: (message: string) => void,
}

// An input field with a button.
export function InputWithButton({ submit, value, onValueUpdated, errorMessage, setErrorMessage } : InputWithButtonProps) {

  const update = (e: React.ChangeEvent<HTMLInputElement>) => onValueUpdated(e.target.value);

  return (
    <Popover
        enforceFocus={false}
        isOpen={errorMessage.length > 0}
        autoFocus={false}
        placement="bottom"
        content={<Callout intent="warning">{errorMessage}</Callout>}>
      <InputGroup
          onKeyUp={chain([() => setErrorMessage(""), onKey("Enter", submit)])}
          value={value}
          className="pt-input"
          placeholder="IP address of new client"
          rightElement={
              <Button disabled={value.trim().length == 0}
                  onClick={submit}
                  minimal={true}
                  intent="primary">Add
              </Button>
            }
          onChange={update} />
    </Popover>

  )
}
