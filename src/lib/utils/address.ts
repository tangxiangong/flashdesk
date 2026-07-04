const DECIMAL_ADDRESS = /^(0|[1-9][0-9]*)$/;
const HEX_ADDRESS = /^0x[0-9a-f]+$/i;

export function parseAddressInput(value: string): number | null {
  const trimmed = value.trim();
  if (!trimmed) return null;

  const radix = trimmed.toLowerCase().startsWith("0x") ? 16 : 10;
  const valid =
    radix === 16 ? HEX_ADDRESS.test(trimmed) : DECIMAL_ADDRESS.test(trimmed);
  if (!valid) return null;

  const parsed = Number.parseInt(trimmed, radix);
  return Number.isSafeInteger(parsed) ? parsed : null;
}
