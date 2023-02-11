Let $lut_{edit}$ be the user supplied custom LUT.  
Let $lut_{final}$ be the color corrected LUT generated by smush_lut.  
Let $x \in [0,1]$ be an RGB color before applying post processing.  
Let $result$ by the screenshot edited by the user from a stage with LUT $lut_{stage}$.  
See the source code for definitions of the post processing functions $f$ and $g$.  
We want $$srgb(g(lut_{final}(f(x)), x)) = lut_{edit}(result)$$
Substituting $result$ we get
$$srgb(g(lut_{final}(f(x)), x)) = lut_{edit}(srgb(g(lut_{stage}(f(x)), x)))$$

Define $g_{x}(y) = g(y, x)$ since $g$ itself is not invertible. We can invert the function $g_{x}$ since $x$ is fixed.  
$$srgb(g_x(lut_{final}(f(x)))) = lut_{edit}(srgb(g_x(lut_{stage}(f(x)))))$$
$g_x$ and $f$ are both invertible, and $linear$ and $srgb$ are inverses of each other, so 
$$lut_{final}(f(x)) = g_x^{-1}(linear(lut_{edit}(srgb(g_x(lut_{stage}(f(x)))))))$$
Let $x_i = f(x)$, so  
$$lut_{final}(x_i) = g_x^{-1}(linear(lut_{edit}(srgb(g_x(lut_{stage}(x_i))))))$$
Now we can construct the lookup table $lut_{final}$ by sampling RGB points $x_i$.
This sampling process introduces some error since we only sample a 16x16x16 grid instead of the full 256x256x256 RGB grid. 
The function evaluations also introduce some errors due to rounding and truncation due to LUTs only use 8 bits per color channel.