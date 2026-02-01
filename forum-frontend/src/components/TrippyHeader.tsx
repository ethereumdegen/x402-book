import { useEffect, useRef } from 'react'
import anime from 'animejs/lib/anime.es.js'

export default function TrippyHeader() {
  const containerRef = useRef<HTMLDivElement>(null)
  const barsRef = useRef<HTMLDivElement>(null)

  useEffect(() => {
    if (!containerRef.current || !barsRef.current) return

    const container = barsRef.current
    container.innerHTML = ''

    // Create a dense grid of tiny bars
    const barCount = 200
    const bars: HTMLDivElement[] = []

    for (let i = 0; i < barCount; i++) {
      const bar = document.createElement('div')
      bar.className = 'micro-bar'
      const width = 2 + Math.random() * 4
      const height = 15 + Math.random() * 40
      bar.style.cssText = `
        position: absolute;
        width: ${width}px;
        height: ${height}px;
        background: #000;
        left: ${Math.random() * 100}%;
        top: ${Math.random() * 100}%;
        transform-origin: center;
      `
      container.appendChild(bar)
      bars.push(bar)
    }

    // Wave animation - bars rise and fall in waves
    anime({
      targets: '.micro-bar',
      translateY: [
        { value: () => anime.random(-30, 30), duration: 1500 },
        { value: () => anime.random(-30, 30), duration: 1500 },
        { value: 0, duration: 1500 }
      ],
      scaleY: [
        { value: () => 0.3 + Math.random() * 1.5, duration: 1200 },
        { value: () => 0.5 + Math.random() * 1.2, duration: 1200 },
        { value: 1, duration: 1200 }
      ],
      rotate: [
        { value: () => anime.random(-45, 45), duration: 2000 },
        { value: () => anime.random(-45, 45), duration: 2000 },
        { value: 0, duration: 2000 }
      ],
      opacity: [
        { value: () => 0.3 + Math.random() * 0.7, duration: 800 },
        { value: () => 0.5 + Math.random() * 0.5, duration: 800 },
        { value: 1, duration: 800 }
      ],
      delay: anime.stagger(15, { from: 'center' }),
      loop: true,
      easing: 'easeInOutQuad'
    })

    // Secondary animation - horizontal drift
    bars.forEach((bar, i) => {
      anime({
        targets: bar,
        translateX: [
          { value: anime.random(-50, 50), duration: 3000 + i * 10 },
          { value: anime.random(-50, 50), duration: 3000 + i * 10 },
          { value: 0, duration: 3000 + i * 10 }
        ],
        easing: 'easeInOutSine',
        loop: true,
        delay: i * 5
      })
    })

    // Create scanning line effect
    const scanLine = document.createElement('div')
    scanLine.style.cssText = `
      position: absolute;
      width: 100%;
      height: 2px;
      background: linear-gradient(90deg, transparent, rgba(0,0,0,0.8), transparent);
      top: 0;
      left: 0;
      z-index: 5;
    `
    container.appendChild(scanLine)

    anime({
      targets: scanLine,
      top: ['0%', '100%'],
      duration: 12000,
      easing: 'linear',
      loop: true
    })

    // Vertical scan line
    const vScanLine = document.createElement('div')
    vScanLine.style.cssText = `
      position: absolute;
      width: 2px;
      height: 100%;
      background: linear-gradient(180deg, transparent, rgba(0,0,0,0.6), transparent);
      top: 0;
      left: 0;
      z-index: 5;
    `
    container.appendChild(vScanLine)

    anime({
      targets: vScanLine,
      left: ['0%', '100%'],
      duration: 18000,
      easing: 'linear',
      loop: true
    })

    return () => {
      anime.remove('.micro-bar')
      anime.remove(scanLine)
      anime.remove(vScanLine)
    }
  }, [])

  return (
    <div
      ref={containerRef}
      style={{
        position: 'relative',
        width: '100%',
        height: '160px',
        borderRadius: '12px',
        overflow: 'hidden',
        marginBottom: '24px',
        background: '#fff',
        boxShadow: '0 4px 30px rgba(0, 0, 0, 0.1), inset 0 0 60px rgba(0, 0, 0, 0.02)'
      }}
    >
      {/* Bars container */}
      <div
        ref={barsRef}
        style={{
          position: 'absolute',
          inset: 0,
          overflow: 'hidden'
        }}
      />

      {/* Subtle gradient overlay */}
      <div
        style={{
          position: 'absolute',
          inset: 0,
          background: 'radial-gradient(ellipse at center, transparent 0%, rgba(255,255,255,0.3) 100%)',
          pointerEvents: 'none',
          zIndex: 10
        }}
      />

      {/* Border */}
      <div
        style={{
          position: 'absolute',
          inset: 0,
          border: '1px solid rgba(0, 0, 0, 0.1)',
          borderRadius: '12px',
          pointerEvents: 'none',
          zIndex: 15
        }}
      />
    </div>
  )
}
