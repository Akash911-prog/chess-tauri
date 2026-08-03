import { useEffect, useRef, useState, type CSSProperties } from "react";

/**
 * SplashScreen
 * Self-contained loading screen — no visibility logic inside.
 * Mount it / unmount it (or wrap in AnimatePresence) from the parent.
 *
 * Two modes:
 *  - Uncontrolled (default): bar fills to 100% over `duration` ms, then calls onComplete.
 *  - Controlled: pass a `progress` number (0-100) yourself and the bar just reflects it.
 */
export interface SplashScreenProps {
    /** main status line */
    label?: string;
    /** ms for auto-fill in uncontrolled mode */
    duration?: number;
    /** 0-100, switches to controlled mode when provided */
    progress?: number;
    /** optional log lines that print one-by-one under the bar */
    messages?: string[];
    /** fires once when internal/controlled progress hits 100 */
    onComplete?: () => void;
}

export default function SplashScreen({
    label = "BOOTING",
    duration = 2200,
    progress,
    messages = [],
    onComplete,
}: SplashScreenProps) {
    const isControlled = progress !== undefined;
    const [internalProgress, setInternalProgress] = useState(0);
    const [visibleLines, setVisibleLines] = useState(0);
    const firedRef = useRef(false);

    const pct = Math.max(
        0,
        Math.min(100, isControlled ? (progress as number) : internalProgress),
    );

    // auto-fill in uncontrolled mode
    useEffect(() => {
        if (isControlled) return;
        let raf: number;
        const start = performance.now();
        const tick = (now: number) => {
            const t = Math.min(1, (now - start) / duration);
            // ease-out so it doesn't feel linear/robotic
            const eased = 1 - Math.pow(1 - t, 3);
            setInternalProgress(eased * 100);
            if (t < 1) raf = requestAnimationFrame(tick);
        };
        raf = requestAnimationFrame(tick);
        return () => cancelAnimationFrame(raf);
    }, [isControlled, duration]);

    // fire onComplete once, whichever mode we're in
    useEffect(() => {
        if (pct >= 100 && !firedRef.current) {
            firedRef.current = true;
            onComplete?.();
        }
        if (pct < 100) firedRef.current = false;
    }, [pct, onComplete]);

    // step log lines in as progress advances
    useEffect(() => {
        if (!messages.length) return;
        const perLine = 100 / messages.length;
        const shouldShow = Math.min(
            messages.length,
            Math.floor(pct / perLine) + (pct > 0 ? 1 : 0),
        );
        setVisibleLines(shouldShow);
    }, [pct, messages.length]);

    const blocks = 24;
    const filledBlocks = Math.round((pct / 100) * blocks);

    return (
        <div style={styles.root} role="status" aria-live="polite">
            <style>{css}</style>
            <div style={styles.scanlines} aria-hidden="true" />

            <div style={styles.frame}>
                <div style={styles.topRow}>
                    <span>SYS://INIT</span>
                    <span>{String(Math.round(pct)).padStart(3, "0")}%</span>
                </div>

                <div style={styles.label}>
                    {label}
                    <span className="splash-cursor">_</span>
                </div>

                <div style={styles.barRow}>
                    <span>[</span>
                    <span style={styles.barTrack}>
                        {Array.from({ length: blocks }).map((_, i) => (
                            <span
                                key={i}
                                style={{
                                    ...styles.barCell,
                                    opacity: i < filledBlocks ? 1 : 0.15,
                                }}
                            >
                                █
                            </span>
                        ))}
                    </span>
                    <span>]</span>
                </div>

                {messages.length > 0 && (
                    <div style={styles.log}>
                        {messages.slice(0, visibleLines).map((line, i) => (
                            <div key={i} style={styles.logLine}>
                                <span style={styles.logPrefix}>&gt;</span>{" "}
                                {line}
                            </div>
                        ))}
                    </div>
                )}
            </div>
        </div>
    );
}

const AMBER = "#e8a33d";
const DIM = "#6b6b6b";
const BG = "#0d0d0d";

const styles: Record<string, CSSProperties> = {
    root: {
        position: "relative",
        width: "100%",
        height: "100%",
        minHeight: "100vh",
        background: BG,
        color: AMBER,
        fontFamily:
            "'JetBrains Mono', 'IBM Plex Mono', ui-monospace, Menlo, monospace",
        display: "flex",
        alignItems: "center",
        justifyContent: "center",
        overflow: "hidden",
    },
    scanlines: {
        position: "absolute",
        inset: 0,
        pointerEvents: "none",
        backgroundImage:
            "repeating-linear-gradient(to bottom, rgba(255,255,255,0.025) 0px, rgba(255,255,255,0.025) 1px, transparent 1px, transparent 3px)",
    },
    frame: {
        width: "min(520px, 86vw)",
        border: `1px solid ${DIM}`,
        padding: "20px 24px",
    },
    topRow: {
        display: "flex",
        justifyContent: "space-between",
        fontSize: 12,
        letterSpacing: "0.08em",
        color: DIM,
        marginBottom: 18,
    },
    label: {
        fontSize: 15,
        letterSpacing: "0.06em",
        marginBottom: 16,
    },
    barRow: {
        display: "flex",
        alignItems: "center",
        gap: 6,
        fontSize: 13,
        letterSpacing: "-0.02em",
    },
    barTrack: {
        display: "inline-flex",
    },
    barCell: {
        transition: "opacity 120ms linear",
    },
    log: {
        marginTop: 18,
        borderTop: "1px solid #2a2a2a",
        paddingTop: 12,
        fontSize: 11,
        color: DIM,
        lineHeight: 1.8,
    },
    logLine: {
        whiteSpace: "nowrap",
        overflow: "hidden",
        textOverflow: "ellipsis",
    },
    logPrefix: {
        color: AMBER,
    },
};

const css = `
.splash-cursor {
  display: inline-block;
  margin-left: 2px;
  animation: splash-blink 1s steps(1) infinite;
}
@keyframes splash-blink {
  0%, 49% { opacity: 1; }
  50%, 100% { opacity: 0; }
}
@media (prefers-reduced-motion: reduce) {
  .splash-cursor { animation: none; opacity: 1; }
}
`;
