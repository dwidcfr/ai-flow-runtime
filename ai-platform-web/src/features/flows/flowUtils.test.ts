import { describe, it, expect } from 'vitest'
import { flowToReactFlow, reactFlowToDraft, autoLayout } from '@/features/flows/flowUtils'
import type { DraftFlowDto } from '@/shared/types'

const sampleFlow: DraftFlowDto = {
  id: 'test_flow',
  version: 1,
  initial: 'start',
  nodes: [
    {
      id: 'start',
      node_type: 'message',
      payload: {},
      transitions: [{ action: 'next', target: 'end' }],
      position: { x: 0, y: 0 },
    },
    {
      id: 'end',
      node_type: 'end',
      payload: {},
      transitions: [],
      position: { x: 0, y: 100 },
    },
  ],
}

describe('flowUtils', () => {
  it('converts flow to react flow nodes and edges', () => {
    const { nodes, edges } = flowToReactFlow(sampleFlow)
    expect(nodes).toHaveLength(2)
    expect(edges).toHaveLength(1)
    expect(edges[0].source).toBe('start')
    expect(edges[0].target).toBe('end')
  })

  it('converts react flow back to draft', () => {
    const { nodes, edges } = flowToReactFlow(sampleFlow)
    const draft = reactFlowToDraft(sampleFlow, nodes, edges)
    expect(draft.nodes).toHaveLength(2)
    expect(draft.nodes[0].transitions[0].target).toBe('end')
  })

  it('auto layout positions nodes', () => {
    const { nodes, edges } = flowToReactFlow(sampleFlow)
    const laid = autoLayout(nodes, edges)
    expect(laid[0].position.y).toBeLessThan(laid[1].position.y)
  })
})
