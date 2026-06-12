function padDatePart(value: number) {
  return String(value).padStart(2, "0");
}

export function formatDateOnly(value: string) {
  const date = new Date(value);
  return [
    date.getFullYear(),
    padDatePart(date.getMonth() + 1),
    padDatePart(date.getDate()),
  ].join("-");
}

export function formatDateTime(value: string) {
  const date = new Date(value);
  return `${formatDateOnly(value)} ${[
    padDatePart(date.getHours()),
    padDatePart(date.getMinutes()),
    padDatePart(date.getSeconds()),
  ].join(":")}`;
}
