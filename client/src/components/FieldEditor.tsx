import { Callout, EditableText, Popover } from "@blueprintjs/core";
import { useRef, useState } from "react";
import { Result } from "../result";

interface FieldEditorProps {
  field: string,
  original: Object,
  onSubmit: (object: Object) => Promise<Result<any>>;
}

export function FieldEditor({ field, original, onSubmit }: FieldEditorProps) {
  const selfRef = useRef<EditableText>(null);

  const [updatedObject, setUpdatedObject] = useState<Object>(clone(original));
  const [errorMessage, setErrorMessage] = useState<string>("");

  return (
    <Popover
        enforceFocus={false}
        isOpen={errorMessage.length > 0}
        autoFocus={false}
        placement="bottom"
        content={<Callout intent="warning">{errorMessage}</Callout>}>

      <EditableText
          ref={selfRef}
          value={Reflect.get(updatedObject, field)}
          onChange={(value: string) => {
              setErrorMessage("");
              const nv = clone(updatedObject);
              Reflect.set(nv, field, value);
              setUpdatedObject(nv);
          }}
          onCancel={_ => {
            setUpdatedObject(original);
            setErrorMessage("");
          }}
          onConfirm={async (value: string) => {
            if (value === Reflect.get(original, field)) {
              setUpdatedObject(original);
              return;
            }
            (await onSubmit(updatedObject)).match({
              ok: _ => {},
              err: msg => {
                setErrorMessage(msg);
                selfRef.current?.toggleEditing();
              }
            })
          }}
      />
    </Popover>
  );
}

export function clone<T>(value: T) {
  return Object.assign(Object.create(Object.getPrototypeOf(value)), value) as T;
}