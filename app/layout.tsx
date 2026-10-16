import type { Metadata, Viewport } from "next"
import "./globals.css"

const base = process.env.NEXT_PUBLIC_BASE_PATH ?? ""

export const metadata: Metadata = {
  title: "Lucky Monkey",
  description: "An isometric jungle network to explore. Proof of concept.",
  icons: { icon: `${base}/icon.svg` },
}

export const viewport: Viewport = {
  themeColor: "#000000",
  width: "device-width",
  initialScale: 1,
  viewportFit: "cover",
}

export default function RootLayout({ children }: Readonly<{ children: React.ReactNode }>) {
  return (
    <html lang="en">
      <body>{children}</body>
    </html>
  )
}
