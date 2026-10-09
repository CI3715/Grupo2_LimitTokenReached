import type {ConnectionStatus} from "./tipes";
import { Theme } from "./tipes";
import {useEffect, useRef} from "react"; 
import type {Particle} from "./tipes";

function NetworkBackground({
  status,
  theme,
}: {
  status: ConnectionStatus;
  theme: Theme;
}) {
  const canvasRef =
    useRef<HTMLCanvasElement>(null);

  useEffect(() => {
    const canvas =
      canvasRef.current;

    if (!canvas) return;

    const ctx =
      canvas.getContext("2d");

    if (!ctx) return;

    let frame = 0;

    let animationFrame = 0;

    let width =
      window.innerWidth;

    let height =
      window.innerHeight;

    const pointer = {
      x: 0,
      y: 0,
    };

    const particles: Particle[] =
      [];

    const palette =
      theme === "dark"
        ? {
            grid: [
              82,
              144,
              255,
            ],
            particle: [
              111,
              177,
              255,
            ],
            connection: [
              92,
              159,
              255,
            ],
          }
        : {
            grid: [
              48,
              105,
              210,
            ],
            particle: [
              48,
              108,
              212,
            ],
            connection: [
              60,
              120,
              220,
            ],
          };

    const getParticleCount =
      () => {
        if (
          window.innerWidth <
          640
        ) {
          return 32;
        }

        if (
          window.innerWidth <
          1024
        ) {
          return 52;
        }

        return 82;
      };

    const createParticles =
      () => {
        particles.length = 0;

        const count =
          getParticleCount();

        for (
          let i = 0;
          i < count;
          i++
        ) {
          const depth =
            Math.random();

          particles.push({
            x:
              Math.random() *
              width,

            y:
              Math.random() *
              height,

            vx:
              (Math.random() -
                0.5) *
              (0.08 +
                depth * 0.16),

            vy:
              (Math.random() -
                0.5) *
              (0.08 +
                depth * 0.16),

            size:
              0.5 +
              depth * 1.3,

            depth,

            phase:
              Math.random() *
              Math.PI *
              2,
          });
        }
      };

    const resize = () => {
      const dpr = Math.min(
        window.devicePixelRatio ||
          1,
        2,
      );

      width =
        window.innerWidth;

      height =
        window.innerHeight;

      canvas.width =
        width * dpr;

      canvas.height =
        height * dpr;

      canvas.style.width =
        `${width}px`;

      canvas.style.height =
        `${height}px`;

      ctx.setTransform(
        dpr,
        0,
        0,
        dpr,
        0,
        0,
      );

      createParticles();
    };

    const handlePointerMove = (
      event: PointerEvent,
    ) => {
      pointer.x =
        event.clientX /
          width -
        0.5;

      pointer.y =
        event.clientY /
          height -
        0.5;
    };

    const drawPerspectiveGrid =
      () => {
        const horizon =
          height * 0.47;

        const bottom =
          height + 120;

        const center =
          width / 2 +
          pointer.x * 25;

        ctx.save();

        ctx.lineWidth = 0.7;

        for (
          let i = -10;
          i <= 10;
          i++
        ) {
          const bottomX =
            center +
            i *
              (width / 8);

          const gradient =
            ctx.createLinearGradient(
              center,
              horizon,
              bottomX,
              bottom,
            );

          const [
            r,
            g,
            b,
          ] = palette.grid;

          gradient.addColorStop(
            0,
            `rgba(${r}, ${g}, ${b}, 0)`,
          );

          gradient.addColorStop(
            1,
            `rgba(${r}, ${g}, ${b}, ${
              theme === "dark"
                ? 0.1
                : 0.07
            })`,
          );

          ctx.strokeStyle =
            gradient;

          ctx.beginPath();

          ctx.moveTo(
            center,
            horizon,
          );

          ctx.lineTo(
            bottomX,
            bottom,
          );

          ctx.stroke();
        }

        const rows = 12;

        for (
          let i = 0;
          i < rows;
          i++
        ) {
          const progress =
            i / rows;

          const eased =
            progress *
            progress;

          const y =
            horizon +
            eased *
              (bottom -
                horizon);

          const [
            r,
            g,
            b,
          ] = palette.grid;

          const opacity =
            theme === "dark"
              ? 0.015 +
                progress *
                  0.055
              : 0.012 +
                progress *
                  0.035;

          ctx.strokeStyle =
            `rgba(${r}, ${g}, ${b}, ${opacity})`;

          ctx.beginPath();

          ctx.moveTo(
            0,
            y,
          );

          ctx.lineTo(
            width,
            y,
          );

          ctx.stroke();
        }

        ctx.restore();
      };

    const drawParticles =
      () => {
        for (
          let i = 0;
          i <
          particles.length;
          i++
        ) {
          const particle =
            particles[i];

          particle.x +=
            particle.vx;

          particle.y +=
            particle.vy;

          if (
            particle.x < -20
          ) {
            particle.x =
              width + 20;
          }

          if (
            particle.x >
            width + 20
          ) {
            particle.x = -20;
          }

          if (
            particle.y < -20
          ) {
            particle.y =
              height + 20;
          }

          if (
            particle.y >
            height + 20
          ) {
            particle.y = -20;
          }

          const parallaxX =
            pointer.x *
            (8 +
              particle.depth *
                18);

          const parallaxY =
            pointer.y *
            (8 +
              particle.depth *
                18);

          const x =
            particle.x +
            parallaxX;

          const y =
            particle.y +
            parallaxY;

          const pulse =
            0.7 +
            Math.sin(
              frame *
                0.012 +
                particle.phase,
            ) *
              0.3;

          let intensity =
            theme === "dark"
              ? 0.16
              : 0.15;

          if (
            status ===
            "loading"
          ) {
            intensity =
              theme === "dark"
                ? 0.3
                : 0.23;
          }

          if (
            status ===
            "connected"
          ) {
            intensity =
              theme === "dark"
                ? 0.38
                : 0.29;
          }

          const [
            pr,
            pg,
            pb,
          ] =
            palette.particle;

          ctx.beginPath();

          ctx.fillStyle =
            `rgba(${pr}, ${pg}, ${pb}, ${
              intensity *
              pulse *
              (0.45 +
                particle.depth)
            })`;

          ctx.arc(
            x,
            y,
            particle.size,
            0,
            Math.PI * 2,
          );

          ctx.fill();

          for (
            let j =
              i + 1;
            j <
            particles.length;
            j++
          ) {
            const other =
              particles[j];

            const ox =
              other.x +
              pointer.x *
                (8 +
                  other.depth *
                    18);

            const oy =
              other.y +
              pointer.y *
                (8 +
                  other.depth *
                    18);

            const dx =
              x - ox;

            const dy =
              y - oy;

            const distance =
              Math.sqrt(
                dx * dx +
                  dy * dy,
              );

            if (
              distance <
              120
            ) {
              const opacity =
                (1 -
                  distance /
                    120) *
                (theme ===
                "dark"
                  ? 0.055
                  : 0.035) *
                (status ===
                "connected"
                  ? 1.4
                  : 1);

              const [
                cr,
                cg,
                cb,
              ] =
                palette.connection;

              ctx.strokeStyle =
                `rgba(${cr}, ${cg}, ${cb}, ${opacity})`;

              ctx.lineWidth =
                0.65;

              ctx.beginPath();

              ctx.moveTo(
                x,
                y,
              );

              ctx.lineTo(
                ox,
                oy,
              );

              ctx.stroke();
            }
          }
        }
      };

    const draw = () => {
      frame += 1;

      ctx.clearRect(
        0,
        0,
        width,
        height,
      );

      drawPerspectiveGrid();

      drawParticles();

      animationFrame =
        requestAnimationFrame(
          draw,
        );
    };

    resize();

    draw();

    window.addEventListener(
      "resize",
      resize,
    );

    window.addEventListener(
      "pointermove",
      handlePointerMove,
    );

    return () => {
      cancelAnimationFrame(
        animationFrame,
      );

      window.removeEventListener(
        "resize",
        resize,
      );

      window.removeEventListener(
        "pointermove",
        handlePointerMove,
      );
    };
  }, [status, theme]);

  return (
    <canvas
      ref={canvasRef}
      className="pointer-events-none fixed inset-0 z-0"
      aria-hidden="true"
    />
  );
}

export default NetworkBackground;