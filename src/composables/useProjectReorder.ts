import { shallowRef } from 'vue'
import type { ProjectBookmark } from '@/types/project'

interface ActivePointerDrag {
  pointerId: number
  projectId: string
  startX: number
  startY: number
  moved: boolean
}

interface DropPosition {
  projectId: string
  side: 'before' | 'after'
}

export function useProjectReorder(
  getProjects: () => ProjectBookmark[],
  onReorder: (projectOrder: string[]) => void,
) {
  const draggingProjectId = shallowRef<string>()
  const dragOverProjectId = shallowRef<string>()
  const dropPosition = shallowRef<DropPosition>()
  let activeDrag: ActivePointerDrag | undefined
  let suppressNextClick = false

  function findDropPosition(event: PointerEvent, draggedProjectId: string): DropPosition | undefined {
    const target = document.elementFromPoint(event.clientX, event.clientY)
      ?.closest<HTMLElement>('[data-project-id]')
    const projectId = target?.dataset.projectId
    if (!target || !projectId || projectId === draggedProjectId
      || !getProjects().some((project) => project.id === projectId)) return undefined

    const rect = target.getBoundingClientRect()
    return {
      projectId,
      side: event.clientY < rect.top + rect.height / 2 ? 'before' : 'after',
    }
  }

  function startDrag(event: PointerEvent, projectId: string) {
    if (event.button !== 0 || getProjects().length < 2) return
    activeDrag = {
      pointerId: event.pointerId,
      projectId,
      startX: event.clientX,
      startY: event.clientY,
      moved: false,
    }
    ;(event.currentTarget as HTMLElement).setPointerCapture(event.pointerId)
  }

  function moveDrag(event: PointerEvent) {
    if (!activeDrag || activeDrag.pointerId !== event.pointerId) return
    if (!activeDrag.moved && Math.hypot(event.clientX - activeDrag.startX, event.clientY - activeDrag.startY) < 6) return

    activeDrag.moved = true
    draggingProjectId.value = activeDrag.projectId
    dropPosition.value = findDropPosition(event, activeDrag.projectId)
    dragOverProjectId.value = dropPosition.value?.projectId
  }

  function endDrag(event: PointerEvent) {
    if (!activeDrag || activeDrag.pointerId !== event.pointerId) return
    const drag = activeDrag
    const hoveredProjectId = document.elementFromPoint(event.clientX, event.clientY)
      ?.closest<HTMLElement>('[data-project-id]')
      ?.dataset.projectId
    const position = hoveredProjectId === drag.projectId
      ? undefined
      : findDropPosition(event, drag.projectId) ?? dropPosition.value
    if (drag.moved && position) {
      const orderedIds = getProjects().map((project) => project.id)
      const from = orderedIds.indexOf(drag.projectId)
      const targetIndex = orderedIds.indexOf(position.projectId)
      if (from >= 0 && targetIndex >= 0) {
        const [moved] = orderedIds.splice(from, 1)
        const adjustedTarget = targetIndex - (from < targetIndex ? 1 : 0)
        const insertionIndex = adjustedTarget + (position.side === 'after' ? 1 : 0)
        orderedIds.splice(insertionIndex, 0, moved)
        if (orderedIds.some((id, index) => id !== getProjects()[index]?.id)) onReorder(orderedIds)
      }
    }

    const element = event.currentTarget as HTMLElement
    if (element.hasPointerCapture(event.pointerId)) element.releasePointerCapture(event.pointerId)
    suppressNextClick = drag.moved
    activeDrag = undefined
    draggingProjectId.value = undefined
    dragOverProjectId.value = undefined
    dropPosition.value = undefined
    if (suppressNextClick) window.setTimeout(() => { suppressNextClick = false }, 0)
  }

  function cancelDrag(event: PointerEvent) {
    if (!activeDrag || activeDrag.pointerId !== event.pointerId) return
    activeDrag = undefined
    draggingProjectId.value = undefined
    dragOverProjectId.value = undefined
    dropPosition.value = undefined
  }

  function handleClick(event: MouseEvent) {
    if (!suppressNextClick) return false
    event.preventDefault()
    event.stopPropagation()
    suppressNextClick = false
    return true
  }

  return { draggingProjectId, dragOverProjectId, dropPosition, startDrag, moveDrag, endDrag, cancelDrag, handleClick }
}
