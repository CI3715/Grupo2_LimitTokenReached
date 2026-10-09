import MoonIcon from "./MoonIcon";
import SunIcon from "./SunIcon";
import type { Theme } from "./tipes";

function ThemeToggle({
  theme,
  onToggle,
}: {
  theme: Theme;
  onToggle: () => void;
}) {
  const nextTheme =
    theme === "dark"
      ? "claro"
      : "oscuro";

  return (
    <button
      type="button"
      onClick={onToggle}
      className="theme-toggle"
      aria-label={`Cambiar a modo ${nextTheme}`}
      title={`Cambiar a modo ${nextTheme}`}
    >
      <span className="theme-toggle-glow" />

      <span className="relative z-10">
        {theme === "dark" ? (
          <SunIcon />
        ) : (
          <MoonIcon />
        )}
      </span>
    </button>
  );
}

export default ThemeToggle;