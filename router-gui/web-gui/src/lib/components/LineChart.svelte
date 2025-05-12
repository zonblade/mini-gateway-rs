<!-- LineChart.svelte -->
<script lang="ts">
    import { onMount, onDestroy } from 'svelte';
    import { writable } from 'svelte/store';
  
    // Define interfaces for our data structures
    interface DataSeries {
      values: number[];
      name?: string;
      color?: string;
    }

    interface DataPoint {
      x: number;
      y: number;
      value: number;
      index: number;
      series: number;
      seriesName: string;
      color: string;
      label: string;
      screenX?: number;
      screenY?: number;
    }

    interface HoveredPoint {
      series: number;
      index: number;
      value: number;
      label: string;
    }

    interface VisibleDataPoints {
      indices: number[];
      values: number[];
    }

    interface PanState {
      x: number;
      y: number;
    }

    interface Size {
      width: number;
      height: number;
    }

    // Component props
    export let data: DataSeries[] = [];
    export let labels: string[] = [];
    export let visiblePoints: number = 12; // Number of data points visible in the viewport
  
    // DOM references
    let canvas: HTMLCanvasElement;
    let ctx: CanvasRenderingContext2D;
    let resizeObserver: ResizeObserver;
    
    // Reactive stores
    const size = writable<Size>({ width: 0, height: 0 });
  
    // Layout parameters
    const padding = 40;
  
    // Transformation state variables
    let scale = 1;
    let offsetX = 0;
    let offsetY = 0;
    let isPanning = false;
    let startPan: PanState;
    let needsDraw = false;
    
    // Zoom configuration
    let autoVerticalZoom = true; // Auto-adjust Y axis based on visible data
    let minVisiblePoints = 5;    // Minimum number of points visible
    let maxVisiblePoints = 50;   // Maximum number of points visible
    
    // Hover state
    let mouseX = 0;
    let mouseY = 0;
    let hoveredPoint: HoveredPoint | null = null;
    let tooltipVisible = false;
    const POINT_RADIUS = 4; // Base radius of data points
    const HOVER_RADIUS = 8; // Hover detection radius
    
    // Tooltip styling
    const tooltipPadding = 8;
    const tooltipRadius = 4;

    /**
     * Constrains offset values within permissible boundaries to prevent
     * excessive panning beyond the scaled chart boundaries.
     * @param {number} w - Chart width
     * @param {number} h - Chart height
     */
    function clampOffsets(w: number, h: number): void {
      // For X, we calculate based on the total data width compared to visible area
      const maxDataPoints = Math.max(...data.map(s => s.values.length), 0);
      const totalPointWidth = w * (maxDataPoints / visiblePoints);
      
      // Allow scrolling to the beginning and end of data
      const maxX = 0;
      const minX = -(totalPointWidth - w);
      
      // For Y, keep the existing constraint
      const maxY = 0;
      const minY = -h * (scale - 1) - 0;
      
      // Apply constraints
      offsetX = Math.min(maxX, Math.max(minX, offsetX));
      offsetY = Math.min(maxY, Math.max(minY, offsetY));
    }
    
    /**
     * Schedules a redraw operation, ensuring that multiple concurrent
     * redraw requests are properly coalesced to optimize rendering performance.
     */
    function scheduleDraw(): void {
      if (needsDraw) return;
      needsDraw = true;
      requestAnimationFrame(() => { 
        draw(); 
        needsDraw = false; 
      });
    }
    
    /**
     * Determines which data points are currently visible in the viewport
     * @param {number} chartW - Width of the chart area
     * @return {Object} Object containing visible point indices and values
     */
    function getVisibleDataPoints(chartW: number): VisibleDataPoints {
      if (!data.length) return { indices: [], values: [] };
      
      const pointWidth = chartW / visiblePoints;
      const leftEdge = -offsetX / pointWidth;
      const rightEdge = leftEdge + visiblePoints;
      
      // Get indices of visible points
      const startIdx = Math.max(0, Math.floor(leftEdge));
      const endIdx = Math.min(
        Math.max(...data.map(s => s.values.length)),
        Math.ceil(rightEdge)
      );
      
      // Get all values from visible points
      const visibleValues: number[] = [];
      data.forEach(series => {
        for (let i = startIdx; i < endIdx && i < series.values.length; i++) {
          visibleValues.push(series.values[i]);
        }
      });
      
      return {
        indices: [startIdx, endIdx],
        values: visibleValues
      };
    }
  
    /**
     * Primary rendering function with distinct transformation contexts for
     * data visualization and axis representation.
     */
    function draw(): void {
      const { width, height } = $size;
      if (!ctx || !width || !height) return;
      ctx.clearRect(0, 0, width, height);
  
      const chartW = width - padding*2;
      const chartH = height - padding*2;
      
      // Calculate min/max Y values - either from all data or only visible data
      let minY: number, maxY: number, rangeY: number;
      
      if (autoVerticalZoom) {
        // Get only values currently visible in the viewport
        const visibleData = getVisibleDataPoints(chartW);
        
        // If we have visible data, use it for Y axis scaling
        if (visibleData.values.length > 0) {
          maxY = Math.max(...visibleData.values, 0);
          minY = Math.min(...visibleData.values, 0);
          
          // Add some padding to avoid points at the very top/bottom
          const padding = (maxY - minY) * 0.1;
          maxY += padding;
          minY -= padding;
        } else {
          // Fallback to all data
          const all = data.flatMap(s => s.values);
          maxY = Math.max(...all, 0);
          minY = Math.min(...all, 0);
        }
      } else {
        // Use all data for Y axis scaling
        const all = data.flatMap(s => s.values);
        maxY = Math.max(...all, 0);
        minY = Math.min(...all, 0);
      }
      
      // Ensure we have a valid range
      rangeY = maxY - minY || 1;
      
      // Determine maximum number of data points in any series
      const maxDataPoints = Math.max(...data.map(s => s.values.length), 0);
      
      // Calculate the width of each data point based on visible points
      const pointWidth = chartW / visiblePoints;
      
      // Calculate the total virtual width of all data points
      const totalWidth = pointWidth * maxDataPoints;
      
      // Static axes lines
      ctx.strokeStyle = '#333'; 
      ctx.lineWidth = 1;
      ctx.beginPath();
      ctx.moveTo(padding, padding);
      ctx.lineTo(padding, padding+chartH);
      ctx.lineTo(padding+chartW, padding+chartH);
      ctx.stroke();
  
      // ============================================================
      // DATA VISUALIZATION WITH COMPLETE BIDIRECTIONAL TRANSFORMATION
      // ============================================================
      ctx.save();
      ctx.beginPath(); 
      ctx.rect(padding, padding, chartW, chartH); 
      ctx.clip();
      ctx.translate(offsetX+padding, offsetY+padding);
      ctx.scale(scale, scale);
  
      // Store point positions for hover detection
      let dataPoints: DataPoint[] = [];
  
      // Render data series
      data.forEach((s, seriesIndex) => {
        ctx.strokeStyle = s.color || '#007bff'; 
        ctx.lineWidth = 2/scale;
        ctx.beginPath();
        
        // Draw lines first
        s.values.forEach((v, i) => {
          // Calculate x position based on fixed point width
          const x = (i * pointWidth);
          const y = chartH - ((v-minY)/rangeY) * chartH;
          i ? ctx.lineTo(x, y) : ctx.moveTo(x, y);
          
          // Store point data for hover detection
          dataPoints.push({
            x, 
            y, 
            value: v,
            index: i,
            series: seriesIndex,
            seriesName: s.name || `Series ${seriesIndex + 1}`,
            color: s.color || '#007bff',
            label: labels[i] || `Point ${i}`
          });
        }); 
        ctx.stroke();
        
        // Now draw circles at each data point
        s.values.forEach((v, i) => {
          // Calculate x position based on fixed point width
          const x = (i * pointWidth);
          const y = chartH - ((v-minY)/rangeY) * chartH;
          
          // Check if this is the hovered point
          const isHovered = hoveredPoint && 
            hoveredPoint.series === seriesIndex && 
            hoveredPoint.index === i;
          
          // Draw circle for data point
          ctx.beginPath();
          ctx.arc(x, y, isHovered ? POINT_RADIUS*1.5/scale : POINT_RADIUS/scale, 0, Math.PI * 2);
          ctx.fillStyle = isHovered ? '#fff' : s.color || '#007bff';
          ctx.fill();
          
          // Add border for better visibility
          ctx.strokeStyle = s.color || '#007bff';
          ctx.lineWidth = isHovered ? 2/scale : 1/scale;
          ctx.stroke();
        });
      });
      
      // Draw tooltip if a point is hovered
      if (hoveredPoint && tooltipVisible) {
        const point = dataPoints.find(p => 
          (hoveredPoint && p.series === hoveredPoint.series) && 
          (hoveredPoint && p.index === hoveredPoint.index)
        );
        
        if (point) {
          // Convert the data point position to screen coordinates
          const screenX = point.x * scale + offsetX + padding;
          const screenY = point.y * scale + offsetY + padding;
          
          // Draw tooltip
          const tooltipText = `${point.label}: ${point.value.toLocaleString()}`;
          const textMetrics = ctx.measureText(tooltipText);
          const tooltipWidth = textMetrics.width + tooltipPadding * 2;
          const tooltipHeight = 24; // Fixed height for simplicity
          
          // Position tooltip above the point
          let tipX = screenX - tooltipWidth / 2;
          let tipY = screenY - tooltipHeight - 10;
          
          // Adjust if tooltip would go off-screen
          tipX = Math.max(padding, Math.min(width - padding - tooltipWidth, tipX));
          tipY = Math.max(padding, tipY);
          
          // Draw tooltip box (in screen coordinates)
          ctx.save();
          ctx.setTransform(1, 0, 0, 1, 0, 0); // Reset transformation
          
          // Background with rounded corners
          ctx.fillStyle = 'rgba(0, 0, 0, 0.7)';
          ctx.beginPath();
          ctx.moveTo(tipX + tooltipRadius, tipY);
          ctx.arcTo(tipX + tooltipWidth, tipY, tipX + tooltipWidth, tipY + tooltipHeight, tooltipRadius);
          ctx.arcTo(tipX + tooltipWidth, tipY + tooltipHeight, tipX, tipY + tooltipHeight, tooltipRadius);
          ctx.arcTo(tipX, tipY + tooltipHeight, tipX, tipY, tooltipRadius);
          ctx.arcTo(tipX, tipY, tipX + tooltipWidth, tipY, tooltipRadius);
          ctx.fill();
          
          // Tooltip text
          ctx.fillStyle = '#fff';
          ctx.font = '12px sans-serif';
          ctx.textAlign = 'center';
          ctx.textBaseline = 'middle';
          ctx.fillText(tooltipText, tipX + tooltipWidth / 2, tipY + tooltipHeight / 2);
          
          ctx.restore();
        }
      }
      
      ctx.restore();
      
      // Save data points for hover detection
      updateDataPoints(dataPoints);
      
      // ==============================================
      // Y-AXIS WITH EXCLUSIVE VERTICAL TRANSFORMATION
      // ==============================================
      ctx.save();
      ctx.beginPath(); 
      ctx.rect(0, padding, padding, chartH); 
      ctx.clip();
      
      // Fixed Y-axis label handling
      ctx.translate(padding, padding);
      
      // Determine optimal tick spacing for Y axis with more constraints
      const minYLabelSpacingPixels = 30; // Minimum spacing between Y labels
      const idealYLabelCount = 10; // Target number of labels for visual consistency
      
      // Calculate data-driven constraints
      const dataRange = maxY - minY;
      const minTickIncrement = calculateMinTickIncrement(dataRange);
      
      // Calculate spacing based on both screen space and data characteristics
      const approxTickSpacing = Math.max(
        minTickIncrement,
        dataRange / idealYLabelCount
      );
      
      // Get a nice rounded tick spacing
      const roundedSpacing = getNiceTickSpacing(approxTickSpacing);
      
      // Calculate tick values covering the entire data range, not just visible portion
      const firstTick = Math.ceil(minY / roundedSpacing) * roundedSpacing;
      const lastTick = Math.floor(maxY / roundedSpacing) * roundedSpacing;
      
      // Check if we'll have too many ticks even with nice spacing
      let actualSpacing = roundedSpacing;
      const estimatedTickCount = Math.ceil((lastTick - firstTick) / actualSpacing) + 1;
      
      // If we'd have too many ticks, adjust by using a larger nice number
      if (estimatedTickCount > 15) {
        actualSpacing = getNiceTickSpacing(approxTickSpacing * 2);
      }
      
      // Draw fixed position ticks with pixel-based positions
      ctx.textAlign = 'right';
      ctx.font = '12px sans-serif';
      
      // Track last label position to prevent overlap
      let lastLabelY = -100; // Start with an offscreen value
      
      for (let tickValue = firstTick; tickValue <= lastTick; tickValue += actualSpacing) {
        // Skip if outside data range
        if (tickValue < minY - 0.00001 || tickValue > maxY + 0.00001) continue;
        
        // Normalize the value to chart coordinates
        const normalizedValue = (tickValue - minY) / rangeY;
        
        // Calculate screen position
        const yScreenPos = chartH * (1 - normalizedValue);
        const transformedY = offsetY + (yScreenPos * scale);
        
        // Skip if it would be outside the visible area
        if (transformedY < 0 || transformedY > chartH) continue;
        
        // Check if this label would overlap with the previous one
        if (Math.abs(transformedY - lastLabelY) < minYLabelSpacingPixels) {
          continue; // Skip this label to prevent overlap
        }
        
        // Draw tick mark
        ctx.strokeStyle = '#333';
        ctx.lineWidth = 1;
        ctx.beginPath();
        ctx.moveTo(-5, transformedY);
        ctx.lineTo(0, transformedY);
        ctx.stroke();
        
        // Format tick value based on magnitude
        let formattedValue: string;
        
        if (Math.abs(tickValue) < 0.00001) {
          // Exact zero special case
          formattedValue = "0";
        } else if (Math.abs(tickValue) < 0.01) {
          // Very small numbers
          formattedValue = tickValue.toExponential(1);
        } else if (Math.abs(tickValue) >= 10000) {
          // Large numbers: use K/M suffixes
          if (Math.abs(tickValue) >= 1000000) {
            formattedValue = (tickValue / 1000000).toFixed(1) + "M";
          } else {
            formattedValue = (tickValue / 1000).toFixed(1) + "K";
          }
        } else if (Number.isInteger(tickValue) || Math.abs(tickValue - Math.round(tickValue)) < 0.00001) {
          // Integers or very close to integers
          formattedValue = Math.round(tickValue).toString();
        } else {
          // Determine optimal decimal places based on tick spacing
          const magnitude = Math.floor(Math.log10(Math.abs(actualSpacing)));
          const decimalPlaces = Math.max(0, Math.min(2, -magnitude + 1));
          formattedValue = tickValue.toFixed(decimalPlaces);
        }
        
        // Draw the label
        ctx.fillText(formattedValue, -10, transformedY + 4);
        
        // Update last label position
        lastLabelY = transformedY;
      }
      
      ctx.restore();
      
      // ==============================================
      // X-AXIS LABELS
      // ==============================================
      // Add X-axis labels if there are any
      if (labels.length > 0) {
        ctx.textAlign = 'center';
        ctx.font = '10px sans-serif';
        ctx.fillStyle = '#333';
        
        // Calculate how many labels to show based on width
        const labelSpacing = Math.ceil(visiblePoints / 6); // Show ~6 labels in visible area
        
        for (let i = 0; i < labels.length; i += labelSpacing) {
          const x = i * pointWidth;
          const y = chartH + 15;
          ctx.fillText(labels[i], x, y);
        }
      }
      
      // Draw zoom info overlay
      ctx.save();
      ctx.fillStyle = 'rgba(0, 0, 0, 0.6)';
      ctx.font = '10px sans-serif';
      ctx.textAlign = 'left';
      
      // Show current zoom level
      const zoomInfo = [
        `Points: ${visiblePoints.toFixed(0)}`,
        `V-Zoom: ${autoVerticalZoom ? 'Auto' : 'Full'}`
      ];
      
      zoomInfo.forEach((text, i) => {
        ctx.fillText(text, padding + 10, padding + 15 + (i * 15));
      });
      
      ctx.restore();
    }
    
    /**
     * Adjust the number of visible points while maintaining the center of the view
     * @param {number} newVisiblePoints - New number of visible points
     * @param {number} focusX - X coordinate to keep centered (default: center of viewport)
     */
    function adjustHorizontalZoom(newVisiblePoints: number, focusX?: number): void {
      if (!ctx) return;
      
      const { width } = $size;
      const chartW = width - padding*2;
      
      // Determine the data point that should stay centered
      focusX = focusX ?? (width / 2);
      
      // Convert focus point to data index
      const pointWidth = chartW / visiblePoints;
      const viewportCenter = ((focusX - padding) - offsetX) / pointWidth;
      
      // Apply zoom constraints
      newVisiblePoints = Math.max(minVisiblePoints, 
                          Math.min(maxVisiblePoints, newVisiblePoints));
      
      // Calculate new offsets to keep the same center point
      const newPointWidth = chartW / newVisiblePoints;
      const newOffsetX = -((viewportCenter * newPointWidth) - (focusX - padding));
      
      // Update state
      visiblePoints = newVisiblePoints;
      offsetX = newOffsetX;
      
      // Ensure offsets are within bounds
      clampOffsets(chartW, $size.height - padding*2);
      scheduleDraw();
    }
    
    /**
     * Toggle vertical auto-zoom mode
     */
    function toggleVerticalZoom(): void {
      autoVerticalZoom = !autoVerticalZoom;
      scheduleDraw();
    }
    
    /**
     * Calculate minimum tick increment based on data range to prevent excessive density
     * @param {number} range - The data range
     * @return {number} Minimum reasonable tick increment
     */
    function calculateMinTickIncrement(range: number): number {
      if (range <= 0) return 1;
      
      // Base calculation on data range magnitude
      const magnitude = Math.pow(10, Math.floor(Math.log10(range)));
      
      // Different scaling factors based on range
      if (range < 10) {
        return magnitude * 0.1; // Very small range (0-10): Use 0.1, 0.2, etc.
      } else if (range < 20) {
        return magnitude * 0.2; // Small range (10-20): Use 0.2, 0.5, 1, 2, etc.
      } else if (range < 100) {
        return magnitude * 0.5; // Medium range (20-100): Use 0.5, 1, 5, 10, etc.
      } else {
        return magnitude * 1; // Large range (100+): Use 1, 2, 5, 10, 20, 50, etc.
      }
    }
    
    /**
     * Helper function to calculate human-readable tick spacing.
     * Returns a "nice" number (1, 2, 5, 10, 20, 50, etc.) for tick intervals.
     * @param {number} rawInterval - The raw calculated interval
     * @return {number} A rounded, human-friendly interval
     */
    function getNiceTickSpacing(rawInterval: number): number {
      if (rawInterval <= 0) return 1;
      
      // Find the magnitude of the interval
      const magnitude = Math.pow(10, Math.floor(Math.log10(rawInterval)));
      const normalized = rawInterval / magnitude;
      
      // Choose a nice multiple based on the normalized value
      let niceMultiple;
      if (normalized < 1.5) {
        niceMultiple = 1;
      } else if (normalized < 3) {
        niceMultiple = 2;
      } else if (normalized < 7) {
        niceMultiple = 5;
      } else {
        niceMultiple = 10;
      }
      
      return niceMultiple * magnitude;
    }
    
    /**
     * Store data points for hover detection
     */
    function updateDataPoints(points: DataPoint[]): void {
      window.requestAnimationFrame(() => {
        // Transform points to screen coordinates for hover detection
        if (ctx) {
          for (const point of points) {
            point.screenX = point.x * scale + offsetX + padding;
            point.screenY = point.y * scale + offsetY + padding;
          }
          // Store for hover detection
          dataPoints = points;
        }
      });
    }
    
    // Store transformed data points for hover detection
    let dataPoints: DataPoint[] = [];
    
    /**
     * Check if mouse is hovering over any data point
     */
    function checkHover(x: number, y: number): void {
      // Find if we're hovering over any point
      const hoverDistance = HOVER_RADIUS * 2; // Distance in pixels to detect hover
      
      let found: HoveredPoint | null = null;
      let minDistance = Infinity;
      
      for (const point of dataPoints) {
        const dx = point.screenX! - x;
        const dy = point.screenY! - y;
        const distance = Math.sqrt(dx*dx + dy*dy);
        
        if (distance < hoverDistance && distance < minDistance) {
          minDistance = distance;
          found = {
            series: point.series,
            index: point.index,
            value: point.value,
            label: point.label
          };
        }
      }
      
      // Update hover state
      if (found) {
        hoveredPoint = found;
        tooltipVisible = true;
      } else if (hoveredPoint) {
        tooltipVisible = false;
        hoveredPoint = null;
      }
      
      // Redraw if hover state changed
      if ((found || hoveredPoint) && needsDraw === false) {
        scheduleDraw();
      }
    }
    
    /**
     * Handle mouse movement for hover detection
     */
    function handleMouseMoveHover(e: MouseEvent): void {
      const rect = canvas.getBoundingClientRect();
      mouseX = e.clientX - rect.left;
      mouseY = e.clientY - rect.top;
      
      checkHover(mouseX, mouseY);
    }
    
    /**
     * Handles mouse wheel events for zooming operations.
     * @param {WheelEvent} e - The wheel event object
     */
    function handleWheel(e: WheelEvent): void {
      e.preventDefault();
      
      // Get mouse position
      const {offsetX: x, offsetY: y, deltaY} = e;
      
      // Check if Shift key is pressed for vertical zoom toggle
      if (e.shiftKey) {
        // Toggle vertical zoom mode
        toggleVerticalZoom();
        return;
      }
      
      // Check if Ctrl/Cmd key is pressed for vertical scale adjustment
      if (e.ctrlKey || e.metaKey) {
        // Adjust vertical scale (existing functionality)
        const prev = scale;
        scale = Math.max(1, deltaY < 0 ? scale * 1.1 : scale / 1.1);
        const dx = x - padding, dy = y - padding;
        offsetX -= dx * (scale/prev - 1);
        offsetY -= dy * (scale/prev - 1);
      } else {
        // Horizontal zoom - adjust number of visible points
        const zoomFactor = deltaY < 0 ? 0.8 : 1.25; // Zoom in/out factor
        const newVisiblePoints = visiblePoints * zoomFactor;
        
        // Zoom toward mouse position
        adjustHorizontalZoom(newVisiblePoints, x);
        return;
      }
      
      // Apply constraints and redraw
      const {width, height} = canvas.getBoundingClientRect();
      clampOffsets(width-padding*2, height-padding*2);
      scheduleDraw();
    }
    
    /**
     * Initiates panning operation on right mouse button press.
     * @param {MouseEvent} e - The mouse event object
     */
    function handleMouseDown(e: MouseEvent): void { 
      if (e.button !== 2) return;
      isPanning = true;
      startPan = {
        x: e.clientX - offsetX,
        y: e.clientY - offsetY
      }; 
    }
    
    /**
     * Updates offsets during active panning operations.
     * @param {MouseEvent} e - The mouse event object
     */
    function handleMouseMove(e: MouseEvent): void { 
      if (!isPanning) {
        handleMouseMoveHover(e);
        return;
      }
      
      offsetX = e.clientX - startPan.x; 
      offsetY = e.clientY - startPan.y; 
      const {width, height} = canvas.getBoundingClientRect(); 
      clampOffsets(width-padding*2, height-padding*2); 
      scheduleDraw(); 
    }
    
    /**
     * Terminates panning operation on right mouse button release.
     * @param {MouseEvent} e - The mouse event object
     */
    function handleMouseUp(e: MouseEvent): void { 
      if (e.button !== 2) return;
      isPanning = false; 
    }
    
    /**
     * Prevents default context menu from appearing on right-click.
     * @param {MouseEvent} e - The context menu event object
     */
    function disableContext(e: MouseEvent): void {
      e.preventDefault();
    }
    
    /**
     * Handles keyboard shortcuts
     * @param {KeyboardEvent} e - The keyboard event
     */
    function handleKeydown(e: KeyboardEvent): void {
      // + key to zoom in horizontally
      if (e.key === '+' || e.key === '=') {
        adjustHorizontalZoom(visiblePoints * 0.8);
      }
      
      // - key to zoom out horizontally
      if (e.key === '-' || e.key === '_') {
        adjustHorizontalZoom(visiblePoints * 1.25);
      }
      
      // 'a' key to toggle auto vertical zoom
      if (e.key === 'a') {
        toggleVerticalZoom();
      }
      
      // 'r' to reset view
      if (e.key === 'r') {
        resetView();
      }
    }
    
    /**
     * Reset view to default
     */
    function resetView(): void {
      if (data.length && data[0].values.length) {
        offsetX = 0;
        offsetY = 0;
        scale = 1;
        visiblePoints = 12;
        scheduleDraw();
      }
    }
  
    // Track previous data reference to detect actual changes
    let prevData: DataSeries[] = data;
    
    // Component lifecycle management
    onMount(() => {
      ctx = canvas.getContext('2d')!; // Non-null assertion as we know canvas is defined
      
      // Initialize ResizeObserver for responsive canvas dimensions
      resizeObserver = new ResizeObserver(entries => {
        for (const entry of entries) { 
          size.set({
            width: entry.contentRect.width,
            height: entry.contentRect.height
          }); 
          canvas.width = entry.contentRect.width; 
          canvas.height = entry.contentRect.height; 
        }
        scheduleDraw();
      });
      
      resizeObserver.observe(canvas.parentElement!);
      
      // Register event listeners
      canvas.addEventListener('wheel', handleWheel, {passive: false});
      canvas.addEventListener('mousedown', handleMouseDown);
      window.addEventListener('mousemove', handleMouseMove);
      window.addEventListener('mouseup', handleMouseUp);
      canvas.addEventListener('contextmenu', disableContext);
      canvas.addEventListener('mouseleave', () => {
        tooltipVisible = false;
        hoveredPoint = null;
        scheduleDraw();
      });
      
      // Add keyboard event listener
      window.addEventListener('keydown', handleKeydown);
      
      // Subscribe to size changes for redraw
      const unsubscribe = size.subscribe(() => scheduleDraw());
      
      // Initialize with default scale and offset for visible points
      const resetViewHandler = () => {
        if (data.length && data[0].values.length) {
          const { width, height } = $size;
          const chartW = width - padding*2;
          
          // Set initial offset to show first 12 points
          offsetX = 0;
          offsetY = 0;
          scale = 1;
          
          scheduleDraw();
        }
      };
      
      // Fixed the unsubscribeData reference issue - it wasn't actually defined or needed
      
      // Cleanup function
      return () => {
        unsubscribe();
        // Removed unsubscribeData() since it wasn't properly defined
        resizeObserver.disconnect();
        canvas.removeEventListener('wheel', handleWheel);
        canvas.removeEventListener('mousedown', handleMouseDown);
        window.removeEventListener('mousemove', handleMouseMove);
        window.removeEventListener('mouseup', handleMouseUp);
        canvas.removeEventListener('contextmenu', disableContext);
        canvas.removeEventListener('mouseleave', () => {});
        window.removeEventListener('keydown', handleKeydown);
      };
    });
    
    // Use a reactive declaration instead of subscription
    $: if (data && data !== prevData) {
      if (data.length) {
        resetView();
      }
      prevData = data;
    }
    
    // Subscribe to data changes for redraw
    $: if (data.length > 0 || labels.length > 0 || visiblePoints) {
      if (ctx) scheduleDraw();
    }
  </script>
  
  <canvas bind:this={canvas} style="display:block;width:100%;height:100%"></canvas>