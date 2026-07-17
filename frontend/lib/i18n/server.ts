import { cookies } from "next/headers";
import { dictionaries, type Locale } from "./dictionaries";

export async function getLocale(): Promise<Locale> {
  const store = await cookies();
  return store.get("locale")?.value === "ar" ? "ar" : "en";
}

export async function getDict() {
  const locale = await getLocale();
  return { locale, dict: dictionaries[locale] };
}
