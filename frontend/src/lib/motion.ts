// Motion design tokens for ArcaneCodex

export const SPRING = {
  gentle: { type: 'spring' as const, stiffness: 120, damping: 14, mass: 1 },
  snappy: { type: 'spring' as const, stiffness: 300, damping: 25, mass: 0.8 },
  bouncy: { type: 'spring' as const, stiffness: 180, damping: 12, mass: 0.6 },
} as const

export const DURATION = {
  micro: 150,
  transition: 250,
  scene: 400,
  stagger: 30,
} as const

export const VARIANTS = {
  fadeIn: {
    initial: { opacity: 0 },
    animate: { opacity: 1 },
    exit: { opacity: 0 },
    transition: DURATION.micro,
  },
  fadeSlideUp: {
    initial: { opacity: 0, y: 16, scale: 0.95 },
    animate: { opacity: 1, y: 0, scale: 1 },
    exit: { opacity: 0, y: -8, scale: 0.95 },
    transition: SPRING.snappy,
  },
  slideFromRight: {
    initial: { opacity: 0, x: 20 },
    animate: { opacity: 1, x: 0 },
    exit: { opacity: 0, x: -20 },
    transition: SPRING.gentle,
  },
  scaleIn: {
    initial: { opacity: 0, scale: 0.9 },
    animate: { opacity: 1, scale: 1 },
    exit: { opacity: 0, scale: 0.85 },
    transition: SPRING.bouncy,
  },
} as const

export const CONTAINER_VARIANTS = {
  hidden: {},
  show: {
    transition: {
      staggerChildren: DURATION.stagger / 1000,
      when: 'beforeChildren' as const,
    },
  },
}

export const ITEM_VARIANTS = {
  hidden: VARIANTS.fadeIn.initial,
  show: VARIANTS.fadeIn.animate,
}
