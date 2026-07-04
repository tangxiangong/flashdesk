export interface HexRow {
  offset: number;
  bytes: string[];
  ascii: string;
}

export function toHexRows(dataHex: string, baseAddress: number): HexRow[] {
  const clean = dataHex.replace(/[^0-9a-fA-F]/g, "");
  const bytes: number[] = [];
  for (let i = 0; i + 1 < clean.length; i += 2) {
    bytes.push(Number.parseInt(clean.slice(i, i + 2), 16));
  }

  const rows: HexRow[] = [];
  for (let i = 0; i < bytes.length; i += 16) {
    const chunk = bytes.slice(i, i + 16);
    rows.push({
      offset: baseAddress + i,
      bytes: chunk.map((b) => b.toString(16).padStart(2, "0")),
      ascii: chunk
        .map((b) => (b >= 32 && b < 127 ? String.fromCharCode(b) : "."))
        .join(""),
    });
  }

  return rows;
}

export function formatAddress(value: number): string {
  return `0x${value.toString(16).padStart(8, "0")}`;
}
