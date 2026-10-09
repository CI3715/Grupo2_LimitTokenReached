function IOSModuleIcon({
  className = "",
}: {
  className?: string;
}) {
  return (
    <div
      className={`ios-module-icon ${className}`}
      aria-hidden="true"
    >
      <span className="ios-module-bar ios-module-bar-top" />
      <span className="ios-module-bar ios-module-bar-bottom" />
    </div>
  );
}

export default IOSModuleIcon;