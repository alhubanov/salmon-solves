import { useSyncExternalStore } from "react";

// The one place the phone breakpoint is written down. App.css repeats the number in its media
// queries — the two describe the same layout, so they have to be changed together.
export const NARROW_QUERY = "(max-width: 900px)";

let mediaQuery = null;

function narrowQuery()
{
  return (mediaQuery ??= window.matchMedia(NARROW_QUERY));
}

function subscribe(onChange)
{
  narrowQuery().addEventListener("change", onChange);
  return () => narrowQuery().removeEventListener("change", onChange);
}

function getSnapshot()
{
  return narrowQuery().matches;
}

// Layout that CSS cannot express on its own: the clue panel's height is set inline from the board's
// pixel size on desktop and must not be on a phone, and the space-bar hint has no meaning there.
// Subscribed rather than held in state, so a resize between render and commit cannot be missed.
export function useIsNarrow()
{
  return useSyncExternalStore(subscribe, getSnapshot);
}
