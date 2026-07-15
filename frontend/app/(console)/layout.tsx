import { Nav } from "@/components/Nav";

export default function ConsoleLayout({
  children,
}: {
  children: React.ReactNode;
}) {
  return (
    <>
      <Nav />
      <main className="mx-auto max-w-[1240px] px-6 py-10">{children}</main>
    </>
  );
}
