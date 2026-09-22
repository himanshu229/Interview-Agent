import { useState, type FormEvent, type KeyboardEvent } from "react";

interface SearchBoxProps {
  onSubmit: (text: string) => void;
  placeholder?: string;
}

export default function SearchBox({ onSubmit, placeholder }: SearchBoxProps) {
  const [value, setValue] = useState("");

  const submit = (e: FormEvent) => {
    e.preventDefault();
    const text = value.trim();
    if (!text) return;
    onSubmit(text);
    setValue("");
  };

  const onKeyDown = (e: KeyboardEvent<HTMLInputElement>) => {
    if (e.key === "Enter" && !e.shiftKey) {
      submit(e);
    }
  };

  return (
    <form className="search-box" onSubmit={submit}>
      <span className="search-box__icon" aria-hidden>
        ⌕
      </span>
      <input
        className="search-box__input"
        value={value}
        onChange={(e) => setValue(e.target.value)}
        onKeyDown={onKeyDown}
        placeholder={placeholder ?? "Ask the AI assistant anything…"}
        autoComplete="off"
        spellCheck
      />
      <button className="search-box__submit" type="submit" disabled={!value.trim()}>
        Ask
      </button>
    </form>
  );
}
